pub struct ItemsOrNone<T> {
    items: Option<Vec<T>>,
}

impl<T> ItemsOrNone<T> {
    pub fn new() -> Self {
        Self { items: None }
    }

    fn creaate_if_needed(&mut self) {
        if self.items.is_none() {
            self.items = Some(Vec::new());
        }
    }

    pub fn push(&mut self, value: T) {
        self.creaate_if_needed();

        let items = self.items.as_mut().unwrap();

        items.push(value);
    }

    pub fn get(&self) -> Option<&[T]> {
        match &self.items {
            Some(result) => Some(result.as_slice()),
            None => None,
        }
    }
}
