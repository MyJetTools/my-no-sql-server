use std::sync::Arc;

use my_http_server::middlewares::controllers::ControllersMiddleware;

use crate::app::AppContext;

pub fn build(app: Arc<AppContext>) -> ControllersMiddleware {
    let mut result = ControllersMiddleware::new();

    let api_controller = super::api::ApiController::new();
    result.register_get_action(Arc::new(api_controller));

    let tables_controller = Arc::new(super::tables::TablesController::new(app.clone()));
    result.register_get_action(tables_controller.clone());
    result.register_post_action(tables_controller.clone());
    result.register_put_action(tables_controller.clone());
    result.register_delete_action(tables_controller);

    let tables_controller = Arc::new(super::tables::TablesController2::new(app.clone()));
    result.register_get_action(tables_controller.clone());
    result.register_post_action(tables_controller);

    let tables_controller = Arc::new(super::tables::MigrationAction::new(app.clone()));
    result.register_post_action(tables_controller);

    let tables_controller = Arc::new(super::tables::CreateIfNotExistsAction::new(app.clone()));
    result.register_post_action(tables_controller);

    let transactions_controller = Arc::new(super::transactions::StartTransactionAction::new(
        app.clone(),
    ));

    result.register_post_action(transactions_controller);

    let transactions_controller = Arc::new(super::transactions::AppendTransactionAction::new(
        app.clone(),
    ));

    result.register_post_action(transactions_controller);

    let transactions_controller = Arc::new(super::transactions::CommitTransactionAction::new(
        app.clone(),
    ));

    result.register_post_action(transactions_controller);

    let transactions_controller = Arc::new(super::transactions::CancelTransactionAction::new(
        app.clone(),
    ));

    result.register_post_action(transactions_controller);

    result.register_post_action(Arc::new(super::multipart::FirstMultipartController::new(
        app.clone(),
    )));

    result.register_post_action(Arc::new(super::multipart::NextMultipartController::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::status::StatusController::new(app.clone())));

    result.register_post_action(Arc::new(super::bulk::BulkDeleteControllerAction::new(
        app.clone(),
    )));

    result.register_post_action(Arc::new(
        super::bulk::CleanAndBulkInsertControllerAction::new(app.clone()),
    ));

    result.register_post_action(Arc::new(
        super::bulk::BlukInsertOrReplaceControllerAction::new(app.clone()),
    ));

    result.register_post_action(Arc::new(super::gc::CleanAndKeepMaxPartitionsAmount::new(
        app.clone(),
    )));

    result.register_post_action(Arc::new(
        super::gc::CleanPartitionAndKepMaxRecordsControllerAction::new(app.clone()),
    ));

    result
}
