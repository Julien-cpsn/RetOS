#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub static DEFAULT_FOREGROUND: Color = Color::YELLOW;
pub static DEFAULT_BACKGROUND: Color = Color::BLACK;

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };
    pub const YELLOW: Color = Color { r: 255, g: 255, b: 0 };
    pub const CYAN: Color = Color { r: 0, g: 255, b: 255 };
    pub const MAGENTA: Color = Color { r: 255, g: 0, b: 255 };
    
    pub fn from_ansi_aligned(byte: u8, foreground: bool) -> Color {
        match byte {
            0 => Color::BLACK,
            1 => Color::RED,
            2 => Color::GREEN,
            3 => Color::YELLOW,
            4 => Color::BLUE,
            5 => Color::MAGENTA,
            6 => Color::CYAN,
            7 => Color::WHITE,
            _ => match foreground {
                true => DEFAULT_FOREGROUND,
                false => DEFAULT_BACKGROUND,
            },
        }
    }
}