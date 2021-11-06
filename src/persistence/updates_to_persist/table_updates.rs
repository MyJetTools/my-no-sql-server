use rust_extensions::date_time::DateTimeAsMicroseconds;

use super::states::{CommonStateData, PartitionsAreUpdatedStateData, TableIsUpdatedStateData};

pub enum TableUpdatesState {
    PartitionsAreUpdated(PartitionsAreUpdatedStateData),
    TableIsUpdated(TableIsUpdatedStateData),
}

pub struct TableUpdates {
    pub state: TableUpdatesState,
}

impl TableUpdates {
    pub fn new_as_partitions_are_updated(
        table_name: String,
        sync_moment: Option<DateTimeAsMicroseconds>,
    ) -> Self {
        let common_state = CommonStateData {
            sync_moment,
            sync_table_attrs: false,
            table_name,
        };
        let state_data = PartitionsAreUpdatedStateData::new(common_state);

        Self {
            state: TableUpdatesState::PartitionsAreUpdated(state_data),
        }
    }

    pub fn new_as_table_is_updated(
        table_name: String,
        sync_moment: Option<DateTimeAsMicroseconds>,
    ) -> Self {
        let common_state = CommonStateData {
            sync_moment,
            sync_table_attrs: false,
            table_name,
        };
        let state_data = TableIsUpdatedStateData::new(common_state);

        Self {
            state: TableUpdatesState::TableIsUpdated(state_data),
        }
    }

    fn get_common_state_mut(&mut self) -> &mut CommonStateData {
        match &mut self.state {
            TableUpdatesState::PartitionsAreUpdated(state) => &mut state.common_state,
            TableUpdatesState::TableIsUpdated(state) => &mut state.common_state,
        }
    }

    pub fn table_attributes_are_updated(&mut self, sync_moment: DateTimeAsMicroseconds) {
        let common_state = self.get_common_state_mut();
        common_state.sync_table_attrs = true;
        common_state.update_sync_moment_if_needed(sync_moment);
    }

    pub fn partitions_are_updated<'a, TKeys: Iterator<Item = &'a String>>(
        &mut self,
        partitions: TKeys,
        sync_moment: DateTimeAsMicroseconds,
    ) {
        match &mut self.state {
            TableUpdatesState::PartitionsAreUpdated(state) => {
                state.new_partitions(partitions);
                state.common_state.update_sync_moment_if_needed(sync_moment);
                return;
            }
            TableUpdatesState::TableIsUpdated(state) => {
                state.common_state.update_sync_moment_if_needed(sync_moment);
            }
        }
    }

    pub fn table_is_updated(&mut self, sync_moment: DateTimeAsMicroseconds) {
        match &mut self.state {
            TableUpdatesState::PartitionsAreUpdated(state) => {
                state.common_state.update_sync_moment_if_needed(sync_moment);

                self.state = TableUpdatesState::TableIsUpdated(TableIsUpdatedStateData::new(
                    state.common_state.clone(),
                ));
            }
            TableUpdatesState::TableIsUpdated(state) => {
                state.common_state.update_sync_moment_if_needed(sync_moment);
            }
        }
    }

    pub fn get_common_state_data(&self) -> &CommonStateData {
        match &self.state {
            TableUpdatesState::PartitionsAreUpdated(state) => &state.common_state,
            TableUpdatesState::TableIsUpdated(state) => &state.common_state,
        }
    }
}
