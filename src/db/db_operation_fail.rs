use crate::db_entity::DbEntityParseFail;

#[derive(Debug)]
pub enum DbOperationFail {
    TableAlreadyExist { table_name: String },
    TableNotFound { table_name: String },

    RecordNotFound,
    OptimisticConcurencyUpdateFails,

    TransactionNotFound { id: String },

    DbEntityParseFail(DbEntityParseFail),
}

impl From<DbEntityParseFail> for DbOperationFail {
    fn from(src: DbEntityParseFail) -> Self {
        Self::DbEntityParseFail(src)
    }
}
