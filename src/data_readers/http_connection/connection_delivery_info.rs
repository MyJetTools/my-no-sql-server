use std::collections::VecDeque;

use my_http_server::{HttpFailResult, HttpOkResult};
use rust_extensions::{TaskCompletion, TaskCompletionAwaiter};

pub struct HttpConnectionDeliveryInfo {
    awaiting_response: Option<TaskCompletion<HttpOkResult, HttpFailResult>>,
    pub payload_to_deliver: VecDeque<HttpOkResult>,
}

impl HttpConnectionDeliveryInfo {
    pub fn new() -> Self {
        Self {
            awaiting_response: None,
            payload_to_deliver: VecDeque::new(),
        }
    }

    pub fn get_task_to_write_response(
        &mut self,
    ) -> Option<TaskCompletion<HttpOkResult, HttpFailResult>> {
        if self.awaiting_response.is_none() {
            return None;
        }

        let mut result = None;
        std::mem::swap(&mut self.awaiting_response, &mut result);
        result
    }

    pub fn issue_task_completion(&mut self) -> TaskCompletionAwaiter<HttpOkResult, HttpFailResult> {
        if self.awaiting_response.is_some() {
            panic!("Task completion is already issued");
        }

        let mut task_completion = TaskCompletion::new();

        let result = task_completion.get_awaiter();

        self.awaiting_response = Some(task_completion);

        result
    }
}
