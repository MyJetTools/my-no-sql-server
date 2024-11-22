use std::sync::Arc;

use my_no_sql_sdk::core::my_json::json_reader::JsonArrayIterator;
use my_no_sql_sdk::core::{db::DbPartition, db_json_entity::DbJsonEntity};

pub fn deserialize(partition_key: &str, raw: &[u8]) -> Result<DbPartition, String> {
    let mut db_partition = DbPartition::new(partition_key.to_string());

    let json_array_iterator = JsonArrayIterator::new(raw);

    if let Err(err) = json_array_iterator {
        return Err(format!(
            "Can not split to array of json objects for partition key: {}. Err: {:?}",
            partition_key, err
        ));
    }

    let json_array_iterator = json_array_iterator.unwrap();

    while let Some(db_entity_json_result) = json_array_iterator.get_next() {
        if let Err(err) = db_entity_json_result {
            return Err(format!("Can not split to array of json objects: {:?}", err));
        }

        let db_entity_json = db_entity_json_result.unwrap();

        match DbJsonEntity::restore_into_db_row(db_entity_json.as_bytes().to_vec()) {
            Ok(db_row) => {
                if db_row.get_partition_key() == partition_key {
                    db_partition.insert_row(Arc::new(db_row));
                } else {
                    println!(
                        "File if partition_key: {} has row with partition_key:{}  and row_key:{}. Skipping Loading Partition",
                        partition_key,
                        db_row.get_partition_key(),
                        db_row.get_row_key()
                    )
                }
            }
            Err(err) => {
                println!("Skipping entity. Reason {:?}", err);
            }
        }
    }

    if db_partition.rows_count() == 0 {
        return Err(format!(
            "No Rows loaded for partition {}. Skipping loading the partition...",
            partition_key
        ));
    }

    Ok(db_partition)
}
