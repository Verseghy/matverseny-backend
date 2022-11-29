use sea_orm::{DbErr, RuntimeErr};
use sqlx::{postgres::PgDatabaseError, Error as SqlxError};
use std::borrow::Cow;

pub trait DatabaseError {
    fn unique_violation(&self, constraint: &str) -> bool;
}

impl DatabaseError for DbErr {
    fn unique_violation(&self, constraint: &str) -> bool {
        if let Some(db_err) = get_database_error(self) {
            if db_err.as_error().is::<PgDatabaseError>() {
                db_err.code() == Some(Cow::Borrowed("23505"))
                    && db_err.constraint() == Some(constraint)
            } else {
                panic!("using not a postgres connection");
            }
        } else {
            false
        }
    }
}

#[allow(clippy::borrowed_box)]
fn get_database_error(err: &DbErr) -> Option<&Box<dyn sqlx::error::DatabaseError + 'static>> {
    match err {
        DbErr::Query(RuntimeErr::SqlxError(SqlxError::Database(db_err))) => Some(db_err),
        DbErr::Exec(RuntimeErr::SqlxError(SqlxError::Database(db_err))) => Some(db_err),
        _ => None,
    }
}
