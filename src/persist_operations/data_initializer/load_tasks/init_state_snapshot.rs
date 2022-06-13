use rust_extensions::date_time::DateTimeAsMicroseconds;

pub struct InitTableStateSnapshot {
    pub name: String,
    pub to_load: usize,
    pub loaded: usize,
    pub list_is_loaded: bool,
    pub init_started: Option<DateTimeAsMicroseconds>,
}

pub struct InitStateSnapshot {
    pub to_load: Vec<InitTableStateSnapshot>,
    pub loading: Vec<InitTableStateSnapshot>,
    pub loaded: Vec<InitTableStateSnapshot>,
}
