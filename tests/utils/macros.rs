#[allow(unused_macros)]
macro_rules! assert_error {
    ($res:expr, $error:expr) => {{
        assert_eq!(Some($res.status()), $error.status());

        let res_json: serde_json::Value = $res.json().await;
        assert_eq!(res_json["code"], $error.code());
    }};
}

#[allow(unused_imports)]
pub(crate) use assert_error;

#[allow(unused_macros)]
macro_rules! assert_ws_handshake {
    ($socket:expr, $user:expr) => {{}};
}

#[allow(unused_macros)]
macro_rules! assert_team_info {
    ($socket:expr, $user:expr) => {{
        use ::tokio_tungstenite::tungstenite::Message;
        $socket
            .send(Message::Text($user.access_token().clone()))
            .await
            .unwrap();
        let message = utils::get_socket_message((&mut $socket).next().await);
        utils::macros::assert_event_type!(message, "TEAM_INFO");
        message
    }};
}

#[allow(unused_imports)]
pub(crate) use assert_team_info;

#[allow(unused_macros)]
macro_rules! assert_event_type {
    ($message:expr, $ty:literal) => {{
        assert!($message.is_object());
        assert!($message["event"].is_string());
        assert_eq!($message["event"].as_str().unwrap(), $ty);
    }};
}

#[allow(unused_imports)]
pub(crate) use assert_event_type;

#[allow(unused_macros)]
macro_rules! enable_logging {
    ($level:ident) => {
        ::tracing_subscriber::fmt()
            .with_max_level(::tracing::level_filters::LevelFilter::$level)
            .with_line_number(true)
            .init();
    };
}

#[allow(unused_imports)]
pub(crate) use enable_logging;

#[allow(unused_macros)]
macro_rules! assert_close_frame {
    ($expr:expr, $code:ident, {$($json:tt)+} $(,)?) => {{
        use ::tokio_tungstenite::tungstenite::{
            protocol::frame::coding::CloseCode,
            Message,
        };

        if let Some(Ok(Message::Close(Some(frame)))) = $expr {
            assert_eq!(frame.code, CloseCode::$code);

            let reason = frame.reason.into_owned();
            let reason: ::serde_json::Value = ::serde_json::from_str(&reason).expect("invalid json");

            ::assert_json_diff::assert_json_eq!(reason, ::serde_json::json!({$($json)+}));
        } else {
            panic!("no or empty close frame");
        }

    }}
}

#[allow(unused_imports)]
pub(crate) use assert_close_frame;

#[allow(unused_macros)]
macro_rules! assert_close_frame_error {
    ($msg:expr, $error:expr) => {{
        use ::tokio_tungstenite::tungstenite::{protocol::frame::coding::CloseCode, Message};

        let Some(Ok(Message::Close(Some(frame)))) = $msg else {
                                                panic!("no or empty close frame");
                                            };

        assert_eq!(frame.code, CloseCode::Error);

        let reason: ::serde_json::Value =
            ::serde_json::from_str(&*frame.reason).expect("invalid json");

        assert_eq!(reason["code"], $error.code());
    }};
}

#[allow(unused_imports)]
pub(crate) use assert_close_frame_error;
