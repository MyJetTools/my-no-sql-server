use std::sync::Arc;

use crate::{
    date_time::MyDateTime,
    db::{DbOperationFail, DbOperationResult, DbRow, DbTable},
};

impl DbTable {
    pub async fn get_rows(
        &self,
        partition_key: Option<&String>,
        row_key: Option<&String>,
    ) -> Result<DbOperationResult, DbOperationFail> {
        let read_access = self.data.read().await;

        if partition_key.is_none() && row_key.is_none() {
            return Ok(read_access.get_all());
        }

        if partition_key.is_some() && row_key.is_none() {
            return Ok(read_access.get_all());
        }

        if partition_key.is_none() && row_key.is_some() {
            let result = read_access.get_by_row_key(row_key.unwrap().as_str());
            return Ok(result);
        }

        let partition_key = partition_key.unwrap();
        let row_key = row_key.unwrap();

        return read_access.get_row(partition_key.as_str(), row_key.as_str());
    }

    pub async fn get_single_partition_multiple_rows(
        &self,
        partition_key: &str,
        row_keys: Vec<String>,
    ) -> Result<DbOperationResult, DbOperationFail> {
        let read_access = self.data.read().await;
        let now = MyDateTime::utc_now();

        let db_partition = read_access.get_partition_and_update_last_access(partition_key, now);

        if db_partition.is_none() {
            return Ok(DbOperationResult::Rows { rows: None });
        }

        let db_partition = db_partition.unwrap();

        let mut db_rows = Vec::new();

        for row_key in &row_keys {
            let db_row = db_partition.get_row_and_update_last_time(row_key, now);

            if let Some(db_row) = db_row {
                db_rows.push(db_row);
            }
        }

        if db_rows.len() == 0 {
            return Ok(DbOperationResult::Rows { rows: None });
        }

        return Ok(DbOperationResult::Rows {
            rows: Some(db_rows),
        });
    }

    //TODO - Unit test it
    pub async fn get_highest_row_and_below(
        &self,
        partition_key: &str,
        row_key: String,
        max_amount: usize,
    ) -> Result<DbOperationResult, DbOperationFail> {
        let read_access = self.data.read().await;
        let now = MyDateTime::utc_now();

        let db_partition = read_access.get_partition_and_update_last_access(partition_key, now);

        if db_partition.is_none() {
            return Ok(DbOperationResult::Rows { rows: None });
        }

        let db_partition = db_partition.unwrap();

        let db_rows = db_partition.get_highest_row_and_below(row_key, now);

        if db_rows.len() == 0 {
            return Ok(DbOperationResult::Rows { rows: None });
        }

        return Ok(DbOperationResult::Rows {
            rows: Some(reverse_and_take(db_rows, max_amount)),
        });
    }
}

fn reverse_and_take(mut src: Vec<Arc<DbRow>>, max_amount: usize) -> Vec<Arc<DbRow>> {
    let mut result = Vec::new();

    for index in src.len() - 1..0 {
        let db_row = src.remove(index);
        result.push(db_row);

        if result.len() >= max_amount {
            break;
        }
    }

    result
}
