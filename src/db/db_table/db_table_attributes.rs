#[derive(Debug, Clone)]
pub struct DbTableAttributes {
    pub persist: bool,
    pub max_partitions_amount: Option<usize>,
}
