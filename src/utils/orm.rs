use sea_orm::{ActiveValue, ConnectionTrait, DbErr, ExecResult, Statement, Value};

pub fn set_option<T>(value: Option<T>) -> ActiveValue<T>
where
    T: Into<Value>,
{
    match value {
        Some(value) => ActiveValue::Set(value),
        None => ActiveValue::NotSet,
    }
}

#[inline(always)]
pub async fn execute_str<C, S>(conn: &C, query: S) -> Result<ExecResult, DbErr>
where
    C: ConnectionTrait,
    S: Into<String>,
{
    conn.execute(Statement::from_string(
        conn.get_database_backend(),
        query.into(),
    ))
    .await
}
