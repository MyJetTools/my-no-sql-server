pub mod clean_partition_and_keep_max_records;
mod keep_max_partitions_amount;
pub use keep_max_partitions_amount::{
    keep_max_partitions_amount, keep_max_partitions_amount_and_expire_db_rows,
};
