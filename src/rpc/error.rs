use core::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    ParseError,
    InvalidRequest,
    MethodNotFound,
    InvalidParams,
    InternalError,
    ServerError(i64),
}

impl ErrorCode {
    pub fn code(&self) -> i64 {
        match *self {
            ErrorCode::ParseError => -32700,
            ErrorCode::InvalidRequest => -32600,
            ErrorCode::MethodNotFound => -32601,
            ErrorCode::InvalidParams => -32602,
            ErrorCode::InternalError => -32603,
            ErrorCode::ServerError(c) => c,
        }
    }

    pub const fn message(&self) -> &'static str {
        match *self {
            ErrorCode::ParseError => "Parse error",
            ErrorCode::InvalidRequest => "Invalid request",
            ErrorCode::MethodNotFound => "Method not found",
            ErrorCode::InvalidParams => "Invalid params",
            ErrorCode::InternalError => "Internal error",
            ErrorCode::ServerError(_) => "Server error",
        }
    }
}

impl Serialize for ErrorCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.code())
    }
}

impl<'a> Deserialize<'a> for ErrorCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        i64::deserialize(deserializer).map(Into::into)
    }
}

impl From<i64> for ErrorCode {
    fn from(code: i64) -> Self {
        match code {
            -32700 => ErrorCode::ParseError,
            -32600 => ErrorCode::InvalidRequest,
            -32601 => ErrorCode::MethodNotFound,
            -32602 => ErrorCode::InvalidParams,
            -32603 => ErrorCode::InternalError,
            _ => ErrorCode::ServerError(code),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RpcError {
    pub code: ErrorCode,
    pub message: Cow<'static, str>,
    pub data: Option<serde_json::Value>,
}

impl RpcError {
    pub const fn new(code: ErrorCode) -> Self {
        RpcError {
            message: Cow::Borrowed(code.message()),
            code,
            data: None,
        }
    }

    pub const fn parse_error() -> Self {
        Self::new(ErrorCode::ParseError)
    }

    pub const fn method_not_found() -> Self {
        Self::new(ErrorCode::MethodNotFound)
    }

    pub const fn invalid_request() -> Self {
        Self::new(ErrorCode::InvalidRequest)
    }

    pub const fn internal_error() -> Self {
        Self::new(ErrorCode::InternalError)
    }

    pub fn invalid_params<M>(message: M) -> Self
    where
        M: Into<String>,
    {
        RpcError {
            code: ErrorCode::InvalidParams,
            message: message.into().into(),
            data: None,
        }
    }

    pub fn internal_error_with<M>(message: M) -> Self
    where
        M: Into<String>,
    {
        RpcError {
            code: ErrorCode::InternalError,
            message: message.into().into(),
            data: None,
        }
    }
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code.message(), self.message)
    }
}
