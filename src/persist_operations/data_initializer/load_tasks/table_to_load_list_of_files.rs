pub struct TableToLoadListOfFiles {
    pub tables_to_init_files: Vec<String>,
    pub table_on_process: Option<String>,
}

impl TableToLoadListOfFiles {
    pub fn new() -> Self {
        Self {
            tables_to_init_files: Vec::new(),
            table_on_process: None,
        }
    }

    pub fn get_next(&mut self) -> Option<String> {
        if self.tables_to_init_files.is_empty() {
            return None;
        }

        let result = self.tables_to_init_files.remove(0);
        self.table_on_process = Some(result.clone());
        Some(result)
    }

    pub fn add_table(&mut self, table_name: String) {
        self.tables_to_init_files.push(table_name);
    }
}
