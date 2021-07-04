use crate::{
    app::AppServices,
    db::{FailOperationResult, OperationResult},
};

pub fn get(app: &AppServices) -> Result<OperationResult, FailOperationResult> {
    let result = OperationResult::Text {
        text: app.metrics.build(),
    };

    Ok(result)
}
