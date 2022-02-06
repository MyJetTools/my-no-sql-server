use std::sync::Arc;

use my_azure_storage_sdk::AzureStorageConnection;
use my_http_server_controllers::controllers::ControllersMiddleware;

use crate::app::AppContext;

pub fn build(
    app: Arc<AppContext>,
    azure_connection: Option<Arc<AzureStorageConnection>>,
) -> ControllersMiddleware {
    let mut result = ControllersMiddleware::new();

    let api_controller = super::api::ApiController::new();
    result.register_get_action(Arc::new(api_controller));

    let tables_controller = Arc::new(super::tables::TablesController::new(
        app.clone(),
        azure_connection,
    ));
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

    result.register_post_action(Arc::new(super::row_controller::InsertRowAction::new(
        app.clone(),
    )));

    result.register_post_action(Arc::new(super::row_controller::InsertOrReplaceAction::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::row_controller::RowCountAction::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::row_controller::RowAction::new(app.clone())));

    result.register_put_action(Arc::new(super::row_controller::RowAction::new(app.clone())));

    result.register_delete_action(Arc::new(super::row_controller::RowAction::new(app.clone())));

    result.register_get_action(Arc::new(
        super::rows_controller::GetHighestRowAndBelowAction::new(app.clone()),
    ));

    result.register_get_action(Arc::new(
        super::rows_controller::GetHighestRowAndBelowAction::new(app.clone()),
    ));

    result.register_post_action(Arc::new(
        super::rows_controller::GetSinglePartitionMultipleRowsAction::new(app.clone()),
    ));

    result.register_get_action(Arc::new(super::logs_controller::LogsByTableAction::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::logs_controller::LogsByProcessAction::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::logs_controller::HomeAction::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::home_controller::IndexAction::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::prometheus_controller::MetricsAction::new(
        app.clone(),
    )));

    result.register_post_action(Arc::new(
        super::data_reader_controller::GreetingAction::new(app.clone()),
    ));

    result.register_post_action(Arc::new(
        super::data_reader_controller::SubscribeAction::new(app.clone()),
    ));

    result.register_post_action(Arc::new(
        super::data_reader_controller::GetChangesAction::new(app.clone()),
    ));

    result.register_post_action(Arc::new(super::data_reader_controller::PingAction::new(
        app.clone(),
    )));

    result
}
