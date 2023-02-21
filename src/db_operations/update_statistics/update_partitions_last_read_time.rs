use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::date_time::DateTimeAsMicroseconds;

pub async fn update_partition_last_read_time(table: &DbTableWrapper, partition: &String) {
    update_partitions_last_read_time(table, [partition].into_iter()).await
}

pub async fn update_partitions_last_read_time<'s, TPartitions: Iterator<Item = &'s String>>(
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
