use rings::model::shared;
use sea_orm::DatabaseConnection;


#[inline]
pub(crate) fn db() -> rings::erx::ResultE<&'static DatabaseConnection> {
    shared()
}
