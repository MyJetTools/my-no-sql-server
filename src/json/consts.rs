pub const ESC_SYMBOL: u8 = '\\' as u8;
pub const DOUBLE_QUOTE: u8 = '"' as u8;
pub const OPEN_BRACKET: u8 = '{' as u8;
pub const CLOSE_BRACKET: u8 = '}' as u8;
pub const DOUBLE_COLUMN: u8 = ':' as u8;

pub const OPEN_ARRAY: u8 = '[' as u8;
pub const CLOSE_ARRAY: u8 = ']' as u8;
pub const COMMA: u8 = ',' as u8;

pub static EMPTY_ARRAY: &'static [u8] = &[OPEN_ARRAY, CLOSE_ARRAY];
