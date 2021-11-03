use std::sync::Arc;

use crate::{db::DbRow, json::JsonArrayBuilder};

pub fn filter_it<'s, TIter: Iterator<Item = &'s Arc<DbRow>>>(
    iterator: TIter,
    limit: Option<usize>,
    skip: Option<usize>,
) -> Vec<u8> {
    let mut array_builder = JsonArrayBuilder::new();

    if let Some(limit) = limit {
        if let Some(skip) = skip {
            for item in iterator.skip(skip).take(limit) {
                array_builder.append_json_object(&item.data);
            }
        } else {
            for item in iterator.take(limit) {
                array_builder.append_json_object(&item.data);
            }
        }
    } else {
        if let Some(skip) = skip {
            for item in iterator.skip(skip) {
                array_builder.append_json_object(&item.data);
            }
        } else {
            for item in iterator {
                array_builder.append_json_object(&item.data);
            }
        }
    }

    array_builder.build()
}
