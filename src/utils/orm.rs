use sea_orm::{ActiveValue, Value};

pub fn set_option<T>(value: Option<T>) -> ActiveValue<T>
where
    T: Into<Value>,
{
    match value {
        Some(value) => ActiveValue::Set(value),
        None => ActiveValue::NotSet,
    }
}
