use std::sync::Arc;

use crate::{
    app::AppServices,
    db::{FailOperationResult, OperationResult},
};

pub fn get(app: Arc<AppServices>) -> Result<OperationResult, FailOperationResult> {
    let result = OperationResult::Text {
        text: app.metrics.build(),
    };

    Ok(result)
}
