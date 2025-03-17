use crate::printer::color::{Color, DEFAULT_BACKGROUND, DEFAULT_FOREGROUND};
use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use core::cmp::max;
use core::fmt;
use font_constants::BACKUP_CHAR;
use noto_sans_mono_bitmap::{
    get_raster,
    get_raster_width,
    FontWeight,
    RasterHeight,
    RasterizedChar,
};
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

    pub const FONT_WEIGHT: FontWeight = FontWeight::Regular;
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
    escape_sequence_level: 0,
    escape_color_indicator: 0,
}));

/// Supports newline characters and implements the `core::fmt::Write` trait.
pub struct Writer {
    pub framebuffer: Option<&'static mut [u8]>,
    pub info: Option<FrameBufferInfo>,
    pub x: usize,
    pub y: usize,
    pub fg_color: Color,
    pub bg_color: Color,
    pub escape_sequence_level: u8,
    pub escape_color_indicator: u8,
}

pub fn set_framebuffer(buffer: &'static mut [u8], info: FrameBufferInfo) {
    let mut writer = WRITER.write();
    writer.framebuffer = Some(buffer);
    writer.info = Some(info);
    writer.clear();
}

/// Returns the raster of the given char or the raster of [`font_constants::BACKUP_CHAR`].
fn get_char_raster(c: char) -> RasterizedChar {
    fn get(c: char) -> Option<RasterizedChar> {
        get_raster(
            c,
            font_constants::FONT_WEIGHT,
            font_constants::CHAR_RASTER_HEIGHT,
        )
    }
    get(c).unwrap_or_else(|| get(BACKUP_CHAR).expect("Should get raster of backup char."))
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
        let reset_escape_sequence = match self.escape_sequence_level {
            0 => match byte {
                // ESC
                0x1B => None,

                _ => Some(self.write_char(char::from(byte))),
            },
            1 => match byte {
                // ESC[
                0x5B => None,

                _ => Some(())
            },
            2 => match byte {
                // Backspace
                0x44 => Some(self.erase_char()),

                // First color first byte
                0x30..=0x39 => {
                    self.escape_color_indicator = byte;
                    None
                },

                _ => Some(())
            }
            3 => match byte {
                // ESC[2K
                0x4B => Some(self.clear_line()),
                // ESC[2J
                0x4A => Some(self.clear()),

                // First color second byte
                0x30..=0x39 => {
                    let color_byte = byte - 0x30;
                    match self.escape_color_indicator {
                        0x33 => self.fg_color = Color::from_ansi_aligned(color_byte, true),
                        0x34 => self.bg_color = Color::from_ansi_aligned(color_byte, false),
                        _ => {}
                    }
                    None
                }

                _ => Some(())
            },
            // First color has been used
            4 if self.escape_color_indicator != 0 => match byte {
                // ESC[__;
                0x3B => None,

                // ESC[__m
                0x6D => Some(()),
                _ => Some(())
            }
            5 => match byte {
                // Second color first byte
                0x30..=0x39 => {
                    self.escape_color_indicator = byte;
                    None
                }
                _ => Some(())
            },
            6 => match byte {
                // Second color second byte
                0x30..=0x39 => {
                    let color_byte = byte - 0x30;
                    match self.escape_color_indicator {
                        0x33 => self.fg_color = Color::from_ansi_aligned(color_byte, true),
                        0x34 => self.bg_color = Color::from_ansi_aligned(color_byte, false),
                        _ => {}
                    }
                    None
                }
                _ => Some(())
            },
            7 => match byte {
                // ESC[__;__m
                0x6D => Some(()),
                _ => Some(())
            }
            _ => Some(self.write_char(char::from(byte)))
        };
        
        match reset_escape_sequence {
            Some(_) => self.escape_sequence_level = 0,
            None => self.escape_sequence_level += 1
        }
    }

    /// Writes a single char to the framebuffer. Takes care of special control characters, such as
    /// newlines and carriage returns.
    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            '\t' => {
                let next_x = self.x + font_constants::CHAR_RASTER_WIDTH * 4;
                if next_x >= self.width() {
                    self.newline();
                }
                let next_y = self.y + font_constants::CHAR_RASTER_HEIGHT.val() * 4 + BORDER_PADDING;
                if next_y >= self.height() {
                    self.clear();
                }
                for _ in 0..4 {
                    self.write_rendered_char(get_char_raster(' '));
                }
            },
            c => {
                let next_x = self.x + font_constants::CHAR_RASTER_WIDTH;
                if next_x >= self.width() {
                    self.newline();
                }
                let next_y = self.y + font_constants::CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
                if next_y >= self.height() {
                    self.clear();
                }
                self.write_rendered_char(get_char_raster(c));
            }
        }
    }

    /// Prints a rendered char into the framebuffer.
    /// Updates `self.x`.
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

        let out_r = ((self.fg_color.r as u16 * alpha as u16 + self.bg_color.r as u16 * inv_alpha as u16) / 255) as u8;
        let out_g = ((self.fg_color.g as u16 * alpha as u16 + self.bg_color.g as u16 * inv_alpha as u16) / 255) as u8;
        let out_b = ((self.fg_color.b as u16 * alpha as u16 + self.bg_color.b as u16 * inv_alpha as u16) / 255) as u8;

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
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_byte(c as u8);
        }
        Ok(())
    }
}