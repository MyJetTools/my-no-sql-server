pub use consts::*;

use crate::db::FailOperationResult;

use super::{consts, db_entity::DbEntity};

pub fn split_to_objects<'t>(data: &'t [u8]) -> Result<Vec<DbEntity<'t>>, FailOperationResult> {
    let mut result: Vec<DbEntity<'t>> = Vec::new();

    let mut object_level: usize = 0;
    let mut inside_string = false;
    let mut escape_mode = false;
    let mut start_index: usize = 0;

    for (i, b) in data.iter().enumerate() {
        if escape_mode {
            escape_mode = false;
            continue;
        }

        match *b {
            ESC_SYMBOL => {
                if inside_string {
                    escape_mode = true;
                }
            }

            DOUBLE_QUOTE => {
                inside_string = !inside_string;
            }

            OPEN_BRACKET => {
                if !inside_string {
                    object_level = object_level + 1;
                    if object_level == 1 {
                        start_index = i
                    }
                }
            }

            CLOSE_BRACKET => {
                if !inside_string {
                    object_level = object_level - 1;
                    if object_level == 0 {
                        let slice = &data[start_index..i + 1];
                        let entity = DbEntity::<'t>::parse(slice)?;
                        result.push(entity);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(result)
}
