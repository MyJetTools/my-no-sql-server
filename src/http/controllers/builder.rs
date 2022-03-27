use std::sync::Arc;

use my_http_server_controllers::controllers::ControllersMiddleware;

use crate::app::AppContext;

pub fn build(app: Arc<AppContext>) -> ControllersMiddleware {
    let mut result = ControllersMiddleware::new();

    let api_controller = super::api::ApiController::new();
    result.register_get_action(Arc::new(api_controller));

    let tables_controller = Arc::new(super::tables_controller::TablesController::new(app.clone()));
    result.register_get_action(tables_controller.clone());

    result.register_put_action(tables_controller.clone());
    result.register_delete_action(tables_controller);

    let crate_table_action = Arc::new(super::tables_controller::CreateTableAction::new(
        app.clone(),
    ));
    result.register_post_action(crate_table_action);

    let update_persist_action = Arc::new(super::tables_controller::UpdatePersistAction::new(
        app.clone(),
    ));

    result.register_post_action(update_persist_action);

    let get_partitions_count_action = Arc::new(
        super::tables_controller::GetPartitionsCountAction::new(app.clone()),
    );

    result.register_get_action(get_partitions_count_action);

    let tables_controller = Arc::new(super::tables_controller::MigrationAction::new(app.clone()));
    result.register_post_action(tables_controller);

    let tables_controller = Arc::new(super::tables_controller::CreateIfNotExistsAction::new(
        app.clone(),
    ));
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

    result.register_get_action(Arc::new(super::status_controller::StatusController::new(
        app.clone(),
    )));

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

    result.register_delete_action(Arc::new(
        super::rows_controller::DeletePartitionsAction::new(app.clone()),
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
