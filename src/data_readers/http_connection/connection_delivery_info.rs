use std::{collections::VecDeque, time::Duration};

use my_http_server::{HttpFailResult, HttpOkResult};
use rust_extensions::{date_time::DateTimeAsMicroseconds, TaskCompletion, TaskCompletionAwaiter};

pub struct AwaitingResponse {
    pub created: DateTimeAsMicroseconds,
    task_completion: TaskCompletion<HttpOkResult, HttpFailResult>,
}

pub struct HttpConnectionDeliveryInfo {
    awaiting_response: Option<AwaitingResponse>,
    pub payload_to_deliver: VecDeque<HttpOkResult>,
}
static MIN_PING_TIMEOUT: Duration = Duration::from_secs(3);

impl HttpConnectionDeliveryInfo {
    pub fn new() -> Self {
        Self {
            awaiting_response: None,
            payload_to_deliver: VecDeque::new(),
        }
    }

    pub fn ping(&mut self, now: DateTimeAsMicroseconds) {
        let ping_me = if let Some(item) = &self.awaiting_response {
            now.duration_since(item.created) >= MIN_PING_TIMEOUT
        } else {
            false
        };

        if !ping_me {
            return;
        }

        if let Some(mut task) = self.get_task_to_write_response() {
            task.set_ok(super::into_http_ok_result::compile_ping_result())
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
        let result = result?;

        result.task_completion.into()
    }

    pub fn issue_task_completion(&mut self) -> TaskCompletionAwaiter<HttpOkResult, HttpFailResult> {
        if self.awaiting_response.is_some() {
            panic!("Task completion is already issued");
        }

        let mut task_completion = TaskCompletion::new();

        let result = task_completion.get_awaiter();

        let awaiting_response = AwaitingResponse {
            created: DateTimeAsMicroseconds::now(),
            task_completion,
        };

        self.awaiting_response = Some(awaiting_response);

        result
    }
}
