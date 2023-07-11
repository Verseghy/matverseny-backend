use sea_orm::{DbErr, RuntimeErr};
use sqlx::{postgres::PgDatabaseError, Error as SqlxError};
use std::borrow::Cow;

pub trait DatabaseError {
    fn unique_violation(&self, constraint: &str) -> bool;
    fn foreign_key_violation(&self, constraint: &str) -> bool;
}

impl DatabaseError for DbErr {
    fn unique_violation(&self, constraint: &str) -> bool {
        is_code_and_constraint(self, "23505", constraint)
    }

    fn foreign_key_violation(&self, constraint: &str) -> bool {
        is_code_and_constraint(self, "23503", constraint)
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

fn is_code_and_constraint(err: &DbErr, code: &str, constraint: &str) -> bool {
    let Some(db_err) = get_database_error(err) else {
        return false;
    };

    if db_err.as_error().is::<PgDatabaseError>() {
        return db_err.code() == Some(Cow::Borrowed(code))
            && db_err.constraint() == Some(constraint);
    }

    panic!("not using a postgres connection");
}
