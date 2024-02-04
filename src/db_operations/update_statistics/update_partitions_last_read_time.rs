use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use my_no_sql_server_core::DbTableWrapper;

pub async fn update_partitions_last_read_time<'s, TPartitions: Iterator<Item = &'s str>>(
    table: &DbTableWrapper,
    partitions: TPartitions,
) {
    let now = DateTimeAsMicroseconds::now();
    let table_access = table.data.read().await;

    for partition_key in partitions {
        if let Some(partition) = table_access.get_partition(partition_key) {
            partition.update_last_read_moment(now);
        }
    }
}
