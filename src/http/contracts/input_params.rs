use my_http_server::{HttpFailResult, QueryString};

use crate::db_sync::DataSynchronizationPeriod;

pub const PARAM_LIMIT: &str = "limit";
pub const PARAM_SKIP: &str = "skip";
pub const PARAM_PARTITION_KEY: &str = "partitionKey";
pub const PARAM_ROW_KEY: &str = "rowKey";
pub const PARAM_TABLE_NAME: &str = "tableName";
pub const PARAM_MAX_PARTITION_AMOUNTS: &str = "maxPartitionsAmount";
pub const PARAM_PERSIST_TABLE: &str = "persist";

pub const DEFAULT_SYNC_PERIOD: DataSynchronizationPeriod = DataSynchronizationPeriod::Sec5;
pub const PERISTS_TABLE_DEFAULT: bool = true;
pub trait MyNoSqlQueryString {
    fn get_sync_period(&self) -> DataSynchronizationPeriod;

    fn get_table_name<'s>(&'s self) -> Result<&'s String, HttpFailResult>;

    fn get_persist_table(&self) -> bool;
    fn get_max_partitions_amount(&self) -> Option<usize>;

    fn get_partition_key_optional<'s>(&'s self) -> Option<&'s String>;
    fn get_partition_key<'s>(&'s self) -> Result<&'s String, HttpFailResult>;

    fn get_row_key_optional<'s>(&'s self) -> Option<&'s String>;
    fn get_row_key<'s>(&'s self) -> Result<&'s String, HttpFailResult>;
}

impl MyNoSqlQueryString for QueryString {
    fn get_partition_key_optional<'s>(&'s self) -> Option<&'s String> {
        self.get_optional_string_parameter(PARAM_PARTITION_KEY)
    }

    fn get_partition_key<'s>(&'s self) -> Result<&'s String, HttpFailResult> {
        self.get_required_string_parameter(PARAM_PARTITION_KEY)
    }

    fn get_row_key_optional<'s>(&'s self) -> Option<&'s String> {
        self.get_optional_string_parameter(PARAM_ROW_KEY)
    }

    fn get_row_key<'s>(&'s self) -> Result<&'s String, HttpFailResult> {
        self.get_required_string_parameter(PARAM_ROW_KEY)
    }

    fn get_table_name<'s>(&'s self) -> Result<&'s String, HttpFailResult> {
        self.get_required_string_parameter(PARAM_TABLE_NAME)
    }

    fn get_persist_table(&self) -> bool {
        match self.get_optional_parameter(PARAM_PERSIST_TABLE) {
            Some(result) => result,
            None => PERISTS_TABLE_DEFAULT,
        }
    }

    fn get_max_partitions_amount(&self) -> Option<usize> {
        self.get_optional_parameter(PARAM_MAX_PARTITION_AMOUNTS)
    }

    fn get_sync_period(&self) -> DataSynchronizationPeriod {
        let sync_period = self.get_optional_string_parameter("syncPeriod");

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
