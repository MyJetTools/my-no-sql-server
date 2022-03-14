mod azure_page_blob_persist_io;
mod blob_errors_handler;
pub mod create_table;
pub mod delete_partition;
pub mod delete_table;
mod partition;
pub mod save_partition;
pub mod save_table_attributes;
mod table;

pub use azure_page_blob_persist_io::AzurePageBlobPersistIo;
