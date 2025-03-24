use crate::printer::buffer::font_constants::{DEFAULT_DIM_FACTOR, DEFAULT_FONT_WEIGHT};
use crate::printer::color::{Color, DEFAULT_BACKGROUND, DEFAULT_FOREGROUND};
use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use core::cmp::max;
use core::fmt;
use font_constants::BACKUP_CHAR;
use noto_sans_mono_bitmap::{get_raster, get_raster_width, FontWeight, RasterHeight, RasterizedChar};
use smallvec::{smallvec, SmallVec};
use spin::{Lazy, RwLock};

/// Additional vertical space between lines
const LINE_SPACING: usize = 2;
/// Additional horizontal space between characters.
const LETTER_SPACING: usize = 0;

/// Padding from the border. Prevent that font is too close to border.
const BORDER_PADDING: usize = 1;

/// Constants for the usage of the [`noto_sans_mono_bitmap`] crate.
mod font_constants {
    use super::*;

    /// Height of each char raster. The font size is ~0.84% of this. Thus, this is the line height that
    /// enables multiple characters to be side-by-side and appear optically in one line in a natural way.
    pub const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;

    /// The width of each single symbol of the mono space font.
    pub const CHAR_RASTER_WIDTH: usize = get_raster_width(FontWeight::Regular, CHAR_RASTER_HEIGHT);

    /// Backup character if a desired symbol is not available by the font.
    /// The '�' character requires the feature "unicode-specials".
    pub const BACKUP_CHAR: char = '�';

    pub const DEFAULT_FONT_WEIGHT: FontWeight = FontWeight::Regular;
    pub const DEFAULT_DIM_FACTOR: u16 = 1;
}

/// A global `Writer` instance.
/// Used by the `print!` and `println!` macros.
pub static WRITER: Lazy<RwLock<Writer>> = Lazy::new(|| RwLock::new(Writer {
    framebuffer: None,
    info: None,
    x: BORDER_PADDING,
    y: BORDER_PADDING,
    fg_color: DEFAULT_FOREGROUND,
    bg_color: DEFAULT_BACKGROUND,
    escape_state: EscapeState::None,
    escape_params: smallvec![],
    current_escape_param: 0,
    font_weight: DEFAULT_FONT_WEIGHT,
    dim_factor: DEFAULT_DIM_FACTOR
}));

/// Supports newline characters and implements the `core::fmt::Write` trait.
pub struct Writer {
    pub framebuffer: Option<&'static mut [u8]>,
    pub info: Option<FrameBufferInfo>,
    pub x: usize,
    pub y: usize,
    pub fg_color: Color,
    pub bg_color: Color,
    pub escape_state: EscapeState,
    pub escape_params: SmallVec<[u16; 8]>,
    pub current_escape_param: u16,
    pub font_weight: FontWeight,
    pub dim_factor: u16,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum EscapeState {
    None,
    Escape,
    CSI,
}

pub fn set_framebuffer(buffer: &'static mut [u8], info: FrameBufferInfo) {
    let mut writer = WRITER.write();
    writer.framebuffer = Some(buffer);
    writer.info = Some(info);
    writer.clear();
}

impl Writer {
    fn newline(&mut self) {
        self.y += font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
        self.carriage_return()
    }

    fn carriage_return(&mut self) {
        self.x = BORDER_PADDING;
    }

    /// Erases all text on the screen. Resets `self.x` and `self.y`.
    pub fn clear(&mut self) {
        self.x = BORDER_PADDING;
        self.y = BORDER_PADDING;

        if let Some(buffer) = self.framebuffer.as_mut() {
            buffer.fill(0);
        }
    }

    pub fn clear_line(&mut self) {
        let width = self.width();
        if let Some(buffer) = self.framebuffer.as_mut() {
            let line_start = self.y * width;
            let line_end = line_start + width;
            buffer[line_start..line_end].fill(0);
        }
        
        //self.y = (self.y - 1) * width;
        self.carriage_return();
    }

    pub fn erase_char(&mut self) {
        if self.x > BORDER_PADDING {
            self.x -= font_constants::CHAR_RASTER_WIDTH + LETTER_SPACING;
        }
        else if self.y > BORDER_PADDING {
            self.y -= font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
            self.x = self.width() - font_constants::CHAR_RASTER_WIDTH;
        }
        else {
            return;
        }

        for y in 0..font_constants::CHAR_RASTER_HEIGHT.val() {
            for x in 0..font_constants::CHAR_RASTER_WIDTH {
                self.write_pixel(self.x + x, self.y + y, 0, 0, 0);
            }
        }
    }

    #[inline]
    fn width(&self) -> usize {
        self.info.unwrap().width
    }

    #[inline]
    fn height(&self) -> usize {
        self.info.unwrap().height
    }

    pub fn write_byte(&mut self, byte: u8) {
        match self.escape_state {
            EscapeState::None => {
                // ESC
                if byte == 0x1B {
                    self.escape_state = EscapeState::Escape;
                } else {
                    self.write_char(char::from(byte));
                }
            },
            EscapeState::Escape => {
                // [
                if byte == 0x5B {
                    self.escape_state = EscapeState::CSI;
                    self.escape_params.clear();
                    self.current_escape_param = 0;
                } else {
                    self.escape_state = EscapeState::None;
                    self.write_char(char::from(byte));
                }
            },
            EscapeState::CSI => {
                match byte {
                    b'0'..=b'9' => {
                        self.current_escape_param = self.current_escape_param * 10 + (byte - b'0') as u16;
                    },
                    b';' => {
                        self.escape_params.push(self.current_escape_param);
                        self.current_escape_param = 0;
                    },
                    // Terminal commands
                    b'K' => {
                        if self.escape_params.len() == 1 && self.escape_params[0] == 2 {
                            self.clear_line();
                        }
                        self.escape_state = EscapeState::None;
                    },
                    b'J' => {
                        if self.escape_params.len() == 1 && self.escape_params[0] == 2 {
                            self.clear();
                        }
                        self.escape_state = EscapeState::None;
                    },
                    // Backspace
                    b'D' => {
                        self.erase_char();
                        self.escape_state = EscapeState::None;
                    },
                    b'm' => {
                        // Push the last parameter if it exists
                        if self.current_escape_param > 0 || self.escape_params.is_empty() {
                            self.escape_params.push(self.current_escape_param);
                        }

                        self.process_sgr_params();
                        self.escape_state = EscapeState::None;
                    },
                    _ => {
                        // Unknown sequence, just reset
                        self.escape_state = EscapeState::None;
                    }
                }
            }
        }
    }


    /// Writes a single char to the framebuffer. Takes care of special control characters, such as
    /// newlines and carriage returns.
    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            '\t' => {
                for _ in 0..4 {
                    self.write_char(' ');
                }
            },
            c => {
                let width = self.width();

                let next_x = self.x + font_constants::CHAR_RASTER_WIDTH;
                if next_x >= width {
                    self.newline();
                }

                let height = self.height();
                let char_height = font_constants::CHAR_RASTER_HEIGHT.val();

                let next_y = self.y + char_height + BORDER_PADDING;
                if next_y >= height {
                    let bytes_per_pixel = self.info.unwrap().bytes_per_pixel;
                    let framebuffer = self.framebuffer.as_mut().unwrap();

                    // Calculate row sizes and offsets
                    let row_bytes = width * bytes_per_pixel;
                    let pixels_to_move = height - char_height - BORDER_PADDING;

                    // Instead of pixel-by-pixel, copy entire rows at once
                    for x in 0..pixels_to_move {
                        let src_offset = ((x + char_height + BORDER_PADDING) * width) * bytes_per_pixel;
                        let dst_offset = (x * width) * bytes_per_pixel;

                        // Copy the entire row at once
                        framebuffer.copy_within(src_offset..src_offset + row_bytes, dst_offset);
                    }

                    // Clear the area at the bottom where new text will go
                    // Instead of pixel-by-pixel, clear entire rows at once
                    let clear_start = pixels_to_move * width * bytes_per_pixel;
                    let clear_end = height * width * bytes_per_pixel;
                    framebuffer[clear_start..clear_end].fill(0);

                    // Reset cursor position
                    self.y -= char_height;
                }
                self.write_rendered_char(self.get_char_raster(c));
            }
        }
    }

    /// Prints a rendered char into the framebuffer.
    /// Updates `self.x`.
    #[inline]
    fn write_rendered_char(&mut self, rendered_char: RasterizedChar) {
        for (y, row) in rendered_char.raster().iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                let (r, g, b) = self.blend(*byte);
                self.write_pixel(self.x + x, self.y + y, r, g, b);
            }
        }
        self.x += rendered_char.width() + LETTER_SPACING;
    }

    #[inline]
    fn blend(&self, alpha: u8) -> (u8, u8, u8) {
        let inv_alpha = 255 - alpha;

        let out_r = (((self.fg_color.r as u16 * alpha as u16 + self.bg_color.r as u16 * inv_alpha as u16) / 255) / self.dim_factor) as u8;
        let out_g = (((self.fg_color.g as u16 * alpha as u16 + self.bg_color.g as u16 * inv_alpha as u16) / 255) / self.dim_factor) as u8;
        let out_b = (((self.fg_color.b as u16 * alpha as u16 + self.bg_color.b as u16 * inv_alpha as u16) / 255) / self.dim_factor) as u8;

        // Pack result
        (out_r, out_g, out_b)
    }

    fn write_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        let pixel_offset = y * self.info.unwrap().stride + x;

        let color = match self.info.unwrap().pixel_format {
            PixelFormat::Rgb => [r, g, b, 0],
            PixelFormat::Bgr => [b, g, r, 0],
            PixelFormat::U8 => [if max(max(r, g), b) > 200 { 0xf } else { 0 }, 0, 0, 0],
            // set a supported (but invalid) pixel format before panicking to avoid a double
            // panic; it might not be readable though
            // if let Some(& mut info) = self.info {
            //     info.as_mut().pixel_format = PixelFormat::Rgb;
            // }
            other => panic!("Pixel format {:?} not supported in logger", other)
        };
        let bytes_per_pixel = self.info.unwrap().bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        if let Some(buffer) = self.framebuffer.as_mut() {
            buffer[byte_offset..(byte_offset + bytes_per_pixel)].copy_from_slice(&color[..bytes_per_pixel]);

            let _ = *&buffer[byte_offset];
        }
    }

    /// Returns the raster of the given char or the raster of [`font_constants::BACKUP_CHAR`].
    #[inline]
    fn get_char_raster(&self, c: char) -> RasterizedChar {
        get_raster(
            c,
            self.font_weight,
            font_constants::CHAR_RASTER_HEIGHT,
        )
            .unwrap_or_else(|| {
                get_raster(
                    BACKUP_CHAR,
                    self.font_weight,
                    font_constants::CHAR_RASTER_HEIGHT,
                )
                    .expect("Should get raster of backup char.")
            })
    }

    fn process_sgr_params(&mut self) {
        if self.escape_params.is_empty() {
            // Reset all attributes if no parameters
            self.reset_attributes();
            return;
        }

        let mut i = 0;
        while i < self.escape_params.len() {
            match self.escape_params[i] {
                // Reset
                0 => self.reset_attributes(),
                // Bold
                1 => self.font_weight = FontWeight::Bold,
                // Dim
                2 => self.dim_factor = 2,
                // Italic
                3 => {},
                // Underline
                4 => {},
                // Blink
                5 => {},
                // Reverse
                7 => {},
                // Hidden
                8 => {},
                // Strike
                9 => {},
                // Fg color
                30..=37 => self.fg_color = Color::from_ansi_aligned((self.escape_params[i] - 30) as u8, true),
                // Bg color
                40..=47 => self.bg_color = Color::from_ansi_aligned((self.escape_params[i] - 40) as u8, false),
                38 | 48 => {
                    // Extended color processing (38;5;n or 48;5;n)
                    if i + 2 < self.escape_params.len() && self.escape_params[i+1] == 5 {
                        let color_byte = self.escape_params[i+2] as u8;
                        if self.escape_params[i] == 38 {
                            self.fg_color = Color::from_ansi_aligned(color_byte, true);
                        } else {
                            self.bg_color = Color::from_ansi_aligned(color_byte, false);
                        }
                        i += 2;
                    }
                },
                _ => {}
            }
            i += 1;
        }
    }

    fn reset_attributes(&mut self) {
        self.fg_color = DEFAULT_FOREGROUND;
        self.bg_color = DEFAULT_BACKGROUND;
        self.font_weight = DEFAULT_FONT_WEIGHT;
        self.dim_factor = DEFAULT_DIM_FACTOR;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_byte(c as u8);
        }
        Ok(())
    }
}