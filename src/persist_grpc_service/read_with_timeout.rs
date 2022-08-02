use std::time::Duration;

use futures::StreamExt;
use tonic::Streaming;

pub async fn read_from_stream<T>(stream: &mut Streaming<T>, timeout: Duration) -> Option<T> {
    let next = stream.next();
    let result = tokio::time::timeout(timeout, next).await.unwrap()?;

    Some(result.unwrap())
}

pub async fn read_from_stream_to_vec<T>(stream: &mut Streaming<T>, timeout: Duration) -> Vec<T> {
    let mut result = Vec::new();
    while let Some(item) = read_from_stream(stream, timeout).await {
        result.push(item);
    }
    result
}
