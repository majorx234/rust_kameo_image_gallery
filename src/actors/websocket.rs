use std::convert::{From, Into};
use std::fmt;
use bytes::{Bytes, BytesMut};
use self::CloseCode::*;


/// Status code used to indicate why an endpoint is closing the `WebSocket`
/// connection.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CloseCode {
    /// Indicates a normal closure, meaning that the purpose for
    /// which the connection was established has been fulfilled.
    Normal,
    /// Indicates that an endpoint is "going away", such as a server
    /// going down or a browser having navigated away from a page.
    Away,
    /// Indicates that an endpoint is terminating the connection due
    /// to a protocol error.
    Protocol,
    /// Indicates that an endpoint is terminating the connection
    /// because it has received a type of data it cannot accept (e.g., an
    /// endpoint that understands only text data MAY send this if it
    /// receives a binary message).
    Unsupported,
    /// Indicates an abnormal closure. If the abnormal closure was due to an
    /// error, this close code will not be used. Instead, the `on_error` method
    /// of the handler will be called with the error. However, if the connection
    /// is simply dropped, without an error, this close code will be sent to the
    /// handler.
    Abnormal,
    /// Indicates that an endpoint is terminating the connection
    /// because it has received data within a message that was not
    /// consistent with the type of the message (e.g., non-UTF-8 [RFC3629]
    /// data within a text message).
    Invalid,
    /// Indicates that an endpoint is terminating the connection
    /// because it has received a message that violates its policy.  This
    /// is a generic status code that can be returned when there is no
    /// other more suitable status code (e.g., Unsupported or Size) or if there
    /// is a need to hide specific details about the policy.
    Policy,
    /// Indicates that an endpoint is terminating the connection
    /// because it has received a message that is too big for it to
    /// process.
    Size,
    /// Indicates that an endpoint (client) is terminating the
    /// connection because it has expected the server to negotiate one or
    /// more extension, but the server didn't return them in the response
    /// message of the WebSocket handshake.  The list of extensions that
    /// are needed should be given as the reason for closing.
    /// Note that this status code is not used by the server, because it
    /// can fail the WebSocket handshake instead.
    Extension,
    /// Indicates that a server is terminating the connection because
    /// it encountered an unexpected condition that prevented it from
    /// fulfilling the request.
    Error,
    /// Indicates that the server is restarting. A client may choose to
    /// reconnect, and if it does, it should use a randomized delay of 5-30
    /// seconds between attempts.
    Restart,
    /// Indicates that the server is overloaded and the client should either
    /// connect to a different IP (when multiple targets exist), or
    /// reconnect to the same IP when a user has performed an action.
    Again,
    Tls,
    Other(u16),
}

impl Into<u16> for CloseCode {
    fn into(self) -> u16 {
        match self {
            Normal => 1000,
            Away => 1001,
            Protocol => 1002,
            Unsupported => 1003,
            Abnormal => 1006,
            Invalid => 1007,
            Policy => 1008,
            Size => 1009,
            Extension => 1010,
            Error => 1011,
            Restart => 1012,
            Again => 1013,
            Tls => 1015,
            Other(code) => code,
        }
    }
}

impl From<u16> for CloseCode {
    fn from(code: u16) -> CloseCode {
        match code {
            1000 => Normal,
            1001 => Away,
            1002 => Protocol,
            1003 => Unsupported,
            1006 => Abnormal,
            1007 => Invalid,
            1008 => Policy,
            1009 => Size,
            1010 => Extension,
            1011 => Error,
            1012 => Restart,
            1013 => Again,
            1015 => Tls,
            _ => Other(code),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
/// Reason for closing the connection
pub struct CloseReason {
    /// Exit code
    pub code: CloseCode,
    /// Optional description of the exit code
    pub description: Option<String>,
}

impl From<CloseCode> for CloseReason {
    fn from(code: CloseCode) -> Self {
        CloseReason {
            code,
            description: None,
        }
    }
}

impl<T: Into<String>> From<(CloseCode, T)> for CloseReason {
    fn from(info: (CloseCode, T)) -> Self {
        CloseReason {
            code: info.0,
            description: Some(info.1.into()),
        }
    }
}

/// `WebSocket` Message
#[derive(Debug, PartialEq)]
pub enum Message {
    /// Text message
    Text(String),
    /// Binary message
    Binary(Bytes),
    /// Ping message
    Ping(String),
    /// Pong message
    Pong(String),
    /// Close message with optional reason
    Close(Option<CloseReason>),
    /// No-op. Useful for actix-net services
    Nop,
}
