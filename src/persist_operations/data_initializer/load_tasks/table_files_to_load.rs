pub struct TableFilesToLoad {
    pub table_name: String,
    pub files_to_load: Vec<String>,
    file_list_is_loaded: bool,
}

impl TableFilesToLoad {
    pub fn new(table_name: String) -> Self {
        Self {
            table_name,
            files_to_load: Vec::new(),
            file_list_is_loaded: false,
        }
    }

    pub fn add_files(&mut self, files: Vec<String>) {
        self.files_to_load.extend(files);
    }

    pub fn file_list_is_loaded(&mut self) {
        self.file_list_is_loaded = true;
    }

    pub fn get_next_file_to_load(&mut self) -> Option<String> {
        if self.files_to_load.len() == 0 {
            return None;
        }

        return Some(self.files_to_load.remove(0));
    }
}
