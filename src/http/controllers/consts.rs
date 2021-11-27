use my_http_utils::QueryString;

use crate::db_sync::DataSynchronizationPeriod;

pub const PARAM_TABLE_NAME: &str = "tableName";
pub const PARAM_LIMIT: &str = "limit";
pub const PARAM_SKIP: &str = "skip";
pub const PARAM_PARTITION_KEY: &str = "partitionKey";
pub const PARAM_ROW_KEY: &str = "rowKey";
pub const PARAM_SYNC_PERIOD: &str = "syncPeriod";

pub const PARAM_PERSIST_TABLE: &str = "persist";
pub const PARAM_MAX_PARTITIONS_AMOUNT: &str = "maxPartitionsAmount";

const DEFAULT_SYNC_PERIOD: DataSynchronizationPeriod = DataSynchronizationPeriod::Sec5;

pub trait MyNoSqlQueryString {
    fn get_sync_period(&self) -> DataSynchronizationPeriod;
}

impl MyNoSqlQueryString for QueryString {
    fn get_sync_period(&self) -> DataSynchronizationPeriod {
        let sync_period = self.get_query_optional_string_parameter(PARAM_SYNC_PERIOD);

        if sync_period.is_none() {
            return DEFAULT_SYNC_PERIOD;
        }

        let sync_period = sync_period.unwrap();

        return match sync_period.as_str() {
            "i" => DataSynchronizationPeriod::Immediately,
            "1" => DataSynchronizationPeriod::Sec1,
            "5" => DataSynchronizationPeriod::Sec5,
            "15" => DataSynchronizationPeriod::Sec15,
            "30" => DataSynchronizationPeriod::Sec30,
            "60" => DataSynchronizationPeriod::Min1,
            "a" => DataSynchronizationPeriod::Asap,
            _ => DEFAULT_SYNC_PERIOD,
        };
    }
}
