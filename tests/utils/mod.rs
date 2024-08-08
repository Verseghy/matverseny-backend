pub mod iam;
pub mod macros;
pub mod prelude;
mod request;
mod response;
pub mod setup;
pub mod team;
pub mod user;

use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

#[allow(unused)]
#[track_caller]
pub fn get_socket_message(
    message: Option<Result<Message, tokio_tungstenite::tungstenite::Error>>,
) -> Value {
    tracing::debug!("socket message: {message:?}");
    if let Some(Ok(Message::Text(message))) = message {
        serde_json::from_str(&message).expect("not json")
    } else {
        panic!("not text");
    }
}

#[allow(unused)]
pub fn uuid() -> String {
    Uuid::new_v4()
        .as_simple()
        .encode_lower(&mut Uuid::encode_buffer())
        .to_owned()
}
