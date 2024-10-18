mod azure;
mod persist_io_operations;
mod sqlite;
mod table_file;
pub use persist_io_operations::{PersistIoOperations, TableListOfFilesUploader};
pub use table_file::TableFile;

pub const TABLE_METADATA_FILE_NAME: &str = ".metadata";
