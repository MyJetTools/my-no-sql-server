use std::collections::HashMap;

use super::common_state_data::CommonStateData;

pub struct PartitionsAreUpdatedStateData {
    pub common_state: CommonStateData,
    pub partitions: HashMap<String, u8>,
}

impl PartitionsAreUpdatedStateData {
    pub fn new(common_state: CommonStateData) -> Self {
        Self {
            common_state: common_state.clone(),
            partitions: HashMap::new(),
        }
    }

    pub fn new_partitions<'a, TKeys: Iterator<Item = &'a String>>(&mut self, partitions: TKeys) {
        for partition_key in partitions.into_iter() {
            if self.partitions.contains_key(partition_key.as_str()) {
                self.partitions.insert(partition_key.to_string(), 0);
            }
        }
    }
}
