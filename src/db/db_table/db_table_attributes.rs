use rust_extensions::date_time::DateTimeAsMicroseconds;

#[derive(Debug, Clone)]
pub struct DbTableAttributes {
    pub persist: bool,
    pub max_partitions_amount: Option<usize>,
    pub created: DateTimeAsMicroseconds,
}
