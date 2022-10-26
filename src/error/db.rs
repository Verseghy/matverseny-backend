use sea_orm::{DbErr, RuntimeErr};
use sqlx::{postgres::PgDatabaseError, Error as SqlxError};

pub trait DatabaseError {
    fn unique_violation(&self, constraint: &str) -> bool;
}

impl DatabaseError for PgDatabaseError {
    fn unique_violation(&self, constraint: &str) -> bool {
        self.code() == "23505" && self.constraint() == Some(constraint)
    }
}

pub trait ToPgError<E> {
    fn to_pg_error(self) -> Result<PgDatabaseError, E>;
}

impl ToPgError<DbErr> for DbErr {
    fn to_pg_error(self) -> Result<PgDatabaseError, DbErr> {
        let mut is_pg_error = false;
        if let DbErr::Query(RuntimeErr::SqlxError(SqlxError::Database(ref error))) = self {
            is_pg_error = error.try_downcast_ref::<PgDatabaseError>().is_some();
        }

        if is_pg_error {
            if let DbErr::Query(RuntimeErr::SqlxError(SqlxError::Database(error))) = self {
                Ok(*error.downcast())
            } else {
                unreachable!()
            }
        } else {
            Err(self)
        }
    }
}
