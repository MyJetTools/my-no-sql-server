use my_http_server::middlewares::controllers::documentation::out_results::HttpResult;

use crate::http::mappers::OperationFailHttpContract;

pub const OPERATION_FAIL_HTTP_STATUS_CODE: u16 = 400;

pub fn op_with_table_is_failed() -> HttpResult {
    HttpResult {
        http_code: OPERATION_FAIL_HTTP_STATUS_CODE,
        nullable: false,
        description: "Operation is failed".to_string(),
        data_type: OperationFailHttpContract::get_http_data_structure().into_http_data_type_array(),
    }
}
