pub struct PersistHistoryDuration {
    data: Vec<usize>,
}

impl PersistHistoryDuration {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn add(&mut self, duration: usize) {
        self.data.push(duration);

        while self.data.len() > 100 {
            self.data.remove(0);
        }
    }

    pub fn get(&self) -> Vec<usize> {
        self.data.clone()
    }
}
