use std::sync::Arc;

use my_http_server::middlewares::controllers::ControllersMiddleware;

use crate::app::AppContext;

pub fn build(app: Arc<AppContext>) -> ControllersMiddleware {
    let mut result = ControllersMiddleware::new();

    let api_controller = super::api::ApiController::new();
    result.register_get_action("/Api/IsAlive", Arc::new(api_controller));

    let tables_controller = Arc::new(super::tables::TablesController::new(app.clone()));
    result.register_get_action("/Tables/List", tables_controller.clone());
    result.register_post_action("/Tables/Create", tables_controller.clone());
    result.register_put_action("/Tables/Clean", tables_controller.clone());
    result.register_delete_action("/Tables/Delete", tables_controller);

    let tables_controller = Arc::new(super::tables::TablesController2::new(app.clone()));
    result.register_get_action("/Tables/PartitionsCount", tables_controller.clone());
    result.register_post_action("/Tables/UpdatePersist", tables_controller);

    let tables_controller = Arc::new(super::tables::MigrationAction::new(app.clone()));
    result.register_post_action("/Tables/MigrateFrom", tables_controller);

    let tables_controller = Arc::new(super::tables::CreateIfNotExistsAction::new(app.clone()));
    result.register_post_action("/Tables/CreateIfNotExists", tables_controller);

    let transactions_controller = Arc::new(super::transactions::StartTransactionAction::new(
        app.clone(),
    ));

    result.register_post_action("/Transactions/Start", transactions_controller);

    let transactions_controller = Arc::new(super::transactions::AppendTransactionAction::new(
        app.clone(),
    ));

    result.register_post_action("/Transactions/Append", transactions_controller);

    let transactions_controller = Arc::new(super::transactions::CommitTransactionAction::new(
        app.clone(),
    ));

    result.register_post_action("/Transactions/Commit", transactions_controller);

    let transactions_controller = Arc::new(super::transactions::CancelTransactionAction::new(
        app.clone(),
    ));

    result.register_post_action("/Transactions/Cancel", transactions_controller);

    result
}
