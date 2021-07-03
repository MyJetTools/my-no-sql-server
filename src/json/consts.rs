pub const ESC_SYMBOL: u8 = '\\' as u8;
pub const DOUBLE_QUOTE: u8 = '"' as u8;
pub const OPEN_BRACKET: u8 = '{' as u8;
pub const CLOSE_BRACKET: u8 = '}' as u8;
pub const DOUBLE_COLUMN: u8 = ':' as u8;

pub const OPEN_ARRAY: u8 = '[' as u8;
pub const CLOSE_ARRAY: u8 = ']' as u8;
pub const COMMA: u8 = ',' as u8;

pub const PARTITION_KEY: &str = "PartitionKey";
pub const ROW_KEY: &str = "RowKey";

pub const TIME_STAMP: &str = "TimeStamp";
pub const EXPIRES: &str = "Expires";

pub static EMPTY_ARRAY: &'static [u8] = &[OPEN_ARRAY, CLOSE_ARRAY];
