pub struct PersistHistoryDuration {
    data: Vec<usize>,
}
const MAX_DATA_LEN: usize = 120;

impl PersistHistoryDuration {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn add(&mut self, duration: usize) {
        self.data.push(duration);

        while self.data.len() >= MAX_DATA_LEN {
            self.data.remove(0);
        }
    }

    pub fn get(&self) -> Vec<usize> {
        self.data.clone()
    }
}
