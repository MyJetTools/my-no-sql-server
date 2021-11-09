use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{db::DbRow, json::JsonArrayBuilder};

pub struct DbRowsFilter<'s, TIter: Iterator<Item = &'s Arc<DbRow>>> {
    pub iterator: TIter,
    limit: Option<usize>,
    skip: Option<usize>,
    skipped: bool,
    yielded: usize,
}

impl<'s, TIter: Iterator<Item = &'s Arc<DbRow>>> DbRowsFilter<'s, TIter> {
    pub fn new(iterator: TIter, limit: Option<usize>, skip: Option<usize>) -> Self {
        Self {
            iterator,
            limit,
            skip,
            skipped: false,
            yielded: 0,
        }
    }
}

impl<'s, TIter: Iterator<Item = &'s Arc<DbRow>>> Iterator for DbRowsFilter<'s, TIter> {
    type Item = &'s Arc<DbRow>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(skip) = self.skip {
            if !self.skipped {
                for _ in 0..skip {
                    self.iterator.next()?;
                }

                self.skipped = true;
            }
        }

        if let Some(limit) = self.limit {
            if self.yielded >= limit {
                return None;
            }
        }

        let result = self.iterator.next()?;

        self.yielded += 1;
        return Some(result);
    }
}

pub fn filter_it<'s, TIter: Iterator<Item = &'s Arc<DbRow>>>(
    iterator: TIter,
    limit: Option<usize>,
    skip: Option<usize>,
    now: DateTimeAsMicroseconds,
) -> Vec<u8> {
    let mut array_builder = JsonArrayBuilder::new();

    for db_row in DbRowsFilter::new(iterator, limit, skip) {
        array_builder.append_json_object(&db_row.data);
        db_row.last_read_access.update(now);
    }

    array_builder.build()
}
