pub mod macro_support {
    pub use assert_json_diff::assert_json_eq;
    pub use futures::{SinkExt, StreamExt};
    pub use tokio_tungstenite::tungstenite::{protocol::frame::coding::CloseCode, Message};
    pub use tracing::level_filters::LevelFilter;
    pub use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
}

#[macro_export]
macro_rules! assert_error {
    ($res:expr, $error:expr) => {{
        assert_eq!(Some($res.status()), $error.status());

        let res_json: serde_json::Value = $res.json().await;
        assert_eq!(res_json["code"], $error.code());
    }};
}

#[macro_export]
macro_rules! assert_ws_handshake {
    ($socket:expr, $user:expr) => {{}};
}

#[macro_export]
macro_rules! assert_team_info {
    ($socket:expr, $user:expr) => {{
        use $crate::macro_support::{StreamExt, SinkExt, Message};

        $socket
            .send(Message::Text(json!({"token": $user.access_token().to_owned()}).to_string()))
            .await
            .unwrap();
        let info = $crate::get_socket_message((&mut $socket).next().await);
        $crate::assert_event_type!(info, "TEAM_INFO");
        let message = $crate::get_socket_message((&mut $socket).next().await);
        $crate::assert_event_type!(message, "UPDATE_TIME");
        info
    }};
}

#[macro_export]
macro_rules! assert_event_type {
    ($message:expr, $ty:literal) => {{
        assert!($message.is_object());
        assert!($message["event"].is_string());
        assert_eq!($message["event"].as_str().unwrap(), $ty);
    }};
}

#[macro_export]
macro_rules! enable_logging {
    ($level:ident) => {{
        use $crate::macro_support::{
            EnvFilter, Layer, LevelFilter, SubscriberExt, SubscriberInitExt,
        };

        let env_filter = EnvFilter::builder()
            .with_default_directive(LevelFilter::$level.into())
            .from_env_lossy();

        ::tracing_subscriber::registry()
            .with(
                ::tracing_subscriber::fmt::layer()
                    .with_test_writer()
                    .with_line_number(true)
                    .with_filter(env_filter),
            )
            .init();
    }};
}

#[macro_export]
macro_rules! assert_close_frame {
    ($expr:expr, $code:ident, {$($json:tt)+} $(,)?) => {{
        use $crate::macro_support::{CloseCode, Message, assert_json_eq};

        if let Some(Ok(Message::Close(Some(frame)))) = $expr {
            assert_eq!(frame.code, CloseCode::$code);

            let reason = frame.reason.into_owned();
            let reason: ::serde_json::Value = ::serde_json::from_str(&reason).expect("invalid json");

            assert_json_eq!(reason, ::serde_json::json!({$($json)+}));
        } else {
            panic!("no or empty close frame");
        }

    }}
}

#[macro_export]
macro_rules! assert_close_frame_error {
    ($msg:expr, $error:expr) => {{
        use $crate::macro_support::{CloseCode, Message};

        let Some(Ok(Message::Close(Some(frame)))) = $msg else {
            panic!("no or empty close frame");
        };

        assert_eq!(frame.code, CloseCode::Error);

        let reason: ::serde_json::Value =
            ::serde_json::from_str(&*frame.reason).expect("invalid json");

        assert_eq!(reason["code"], $error.code());
    }};
}
