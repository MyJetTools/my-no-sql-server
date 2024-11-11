use std::sync::Arc;

use my_http_server::controllers::ControllersMiddleware;

use crate::app::AppContext;

pub fn build(app: &Arc<AppContext>) -> ControllersMiddleware {
    let mut result = ControllersMiddleware::new(None, None);

    let api_controller = super::api::IsAliveAction::new();
    result.register_get_action(Arc::new(api_controller));

    result.register_get_action(Arc::new(super::tables_controller::GetListAction::new(
        app.clone(),
    )));

    result.register_put_action(Arc::new(super::tables_controller::CleanTableAction::new(
        app.clone(),
    )));

    result.register_delete_action(Arc::new(super::tables_controller::DeleteTableAction::new(
        app.clone(),
    )));

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

    let get_table_size_action = Arc::new(super::tables_controller::GetTableSizeAction::new(
        app.clone(),
    ));

    result.register_get_action(get_table_size_action);

    result.register_post_action(Arc::new(super::tables_controller::MigrationAction::new(
        app.clone(),
    )));

    result.register_post_action(Arc::new(
        super::tables_controller::CreateIfNotExistsAction::new(app.clone()),
    ));

    result.register_post_action(Arc::new(super::transactions::StartTransactionAction::new(
        app.clone(),
    )));

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

    result.register_post_action(Arc::new(super::multipart::FirstMultipartAction::new(
        app.clone(),
    )));

    result.register_post_action(Arc::new(super::multipart::NextMultipartAction::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::status_controller::StatusAction::new(
        app.clone(),
    )));

    result.register_post_action(Arc::new(super::bulk::BulkDeleteAction::new(app.clone())));

    result.register_post_action(Arc::new(super::bulk::CleanAndBulkInsertAction::new(
        app.clone(),
    )));

    result.register_post_action(Arc::new(super::bulk::BulkInsertOrReplaceAction::new(
        app.clone(),
    )));

    result.register_post_action(Arc::new(
        super::gc_controller::CleanAndKeepMaxPartitionsAmountAction::new(app.clone()),
    ));

    result.register_post_action(Arc::new(
        super::gc_controller::CleanPartitionAndKepMaxRecordsControllerAction::new(app.clone()),
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

    result.register_get_action(Arc::new(super::row_controller::GetRowsAction::new(
        app.clone(),
    )));

    result.register_put_action(Arc::new(super::row_controller::ReplaceRowAction::new(
        app.clone(),
    )));

    result.register_delete_action(Arc::new(super::row_controller::DeleteRowAction::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(
        super::rows_controller::GetHighestRowAndBelowAction::new(app.clone()),
    ));

    result.register_post_action(Arc::new(
        super::rows_controller::GetSinglePartitionMultipleRowsAction::new(app.clone()),
    ));

    result.register_delete_action(Arc::new(
        super::rows_controller::DeletePartitionsAction::new(app.clone()),
    ));

    /*
       result.register_get_action(Arc::new(super::logs_controller::GetFatalErrorsAction::new(
           app.clone(),
       )));

       result.register_get_action(Arc::new(super::logs_controller::GetLogsByTableAction::new(
           app.clone(),
       )));

       result.register_get_action(Arc::new(super::logs_controller::SelectTableAction::new(
           app.clone(),
       )));


    result.register_get_action(Arc::new(
        super::logs_controller::GetLogsByProcessAction::new(app.clone()),
    ));

    result.register_get_action(Arc::new(super::logs_controller::SelectProcessAction::new()));

    result.register_get_action(Arc::new(super::logs_controller::HomeAction::new(
        app.clone(),
    )));
    */

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

    let force_persist_action = super::persist_controller::ForcePersistAction::new(app.clone());

    result.register_get_action(Arc::new(
        super::debug_controller::GetRowStatisticsAction::new(app.clone()),
    ));

    result.register_post_action(Arc::new(force_persist_action));

    // Partitions Controller

    result.register_get_action(Arc::new(super::partitions::GetPartitionsAction::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::partitions::GetPartitionsCountAction::new(
        app.clone(),
    )));

    // Backup controller

    result.register_get_action(Arc::new(super::backup_controller::DownloadAction::new(
        app.clone(),
    )));

    result
}
