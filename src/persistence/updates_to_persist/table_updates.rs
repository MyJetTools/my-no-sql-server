use rust_extensions::date_time::DateTimeAsMicroseconds;

use super::states::{CommonStateData, PartitionsAreUpdatedStateData, TableIsUpdatedStateData};

pub enum TableUpdatesState {
    Empty(CommonStateData),
    PartitionsAreUpdated(PartitionsAreUpdatedStateData),
    TableIsUpdated(TableIsUpdatedStateData),
}

pub struct TableUpdates {
    state: TableUpdatesState,
}

impl TableUpdates {
    pub fn new(table_name: String) -> Self {
        Self {
            state: TableUpdatesState::Empty(CommonStateData::new(table_name, None, false)),
        }
    }

    fn get_common_state(&mut self) -> &mut CommonStateData {
        match &mut self.state {
            TableUpdatesState::Empty(state) => state,
            TableUpdatesState::PartitionsAreUpdated(state) => &mut state.common_state,
            TableUpdatesState::TableIsUpdated(state) => &mut state.common_state,
        }
    }

    pub fn table_attributes_are_updated(&mut self, sync_moment: DateTimeAsMicroseconds) {
        let common_state = self.get_common_state();
        common_state.sync_table_attrs = true;
        common_state.update_sync_moment_if_needed(sync_moment);
    }

    pub fn partitions_are_updated<'a, TKeys: Iterator<Item = &'a String>>(
        &mut self,
        partitions: TKeys,
        sync_moment: DateTimeAsMicroseconds,
    ) {
        match &mut self.state {
            TableUpdatesState::Empty(common_state) => {
                let mut state = PartitionsAreUpdatedStateData::new(common_state.clone());
                state.new_partitions(partitions);
                self.state = TableUpdatesState::PartitionsAreUpdated(state);
            }
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
            TableUpdatesState::Empty(common_state) => {
                common_state.update_sync_moment_if_needed(sync_moment);

                self.state = TableUpdatesState::TableIsUpdated(TableIsUpdatedStateData::new(
                    common_state.clone(),
                ))
            }
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

    pub fn get_update_state(
        &mut self,
        now: DateTimeAsMicroseconds,
        is_shutting_down: bool,
    ) -> Option<TableUpdatesState> {
        match &mut self.state {
            TableUpdatesState::Empty(_) => None,
            TableUpdatesState::PartitionsAreUpdated(state) => {
                let sync_it = if let Some(sync_moment) = &state.common_state.sync_moment {
                    is_shutting_down || now.unix_microseconds >= sync_moment.unix_microseconds
                } else {
                    true
                };

                if !sync_it {
                    return None;
                }

                let mut result =
                    TableUpdatesState::Empty(CommonStateData::copy_new(&state.common_state));

                std::mem::swap(&mut result, &mut self.state);

                Some(result)
            }
            TableUpdatesState::TableIsUpdated(state) => {
                let sync_it = if let Some(sync_moment) = &state.common_state.sync_moment {
                    now.unix_microseconds >= sync_moment.unix_microseconds
                } else {
                    true
                };

                if !sync_it {
                    return None;
                }

                let mut result =
                    TableUpdatesState::Empty(CommonStateData::copy_new(&state.common_state));

                std::mem::swap(&mut result, &mut self.state);

                Some(result)
            }
        }
    }
}
