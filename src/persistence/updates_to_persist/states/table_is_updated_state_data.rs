use super::common_state_data::CommonStateData;

pub struct TableIsUpdatedStateData {
    pub common_state: CommonStateData,
}

impl TableIsUpdatedStateData {
    pub fn new(common_state: CommonStateData) -> Self {
        Self {
            common_state: common_state.clone(),
        }
    }
}
