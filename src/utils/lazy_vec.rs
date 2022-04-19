pub struct LazyVec<T> {
    items: Option<Vec<T>>,
}

impl<T> LazyVec<T> {
    pub fn new() -> Self {
        Self { items: None }
    }

    fn get_items(&mut self) -> &mut Vec<T> {
        if self.items.is_none() {
            self.items = Some(Vec::new());
        }

        self.items.as_mut().unwrap()
    }

    pub fn push(&mut self, item: T) {
        self.get_items().push(item);
    }

    pub fn get_result(self) -> Option<Vec<T>> {
        self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_none()
    }
}
