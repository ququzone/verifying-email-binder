use super::{
    error::RpcError,
    request::{Id, Version},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ResponseResult {
    #[serde(rename = "result")]
    Success(serde_json::Value),

    #[serde(rename = "error")]
    Error(RpcError),
}

impl ResponseResult {
    pub fn success<S>(content: S) -> Self
    where
        S: Serialize + 'static,
    {
        ResponseResult::Success(serde_json::to_value(&content).unwrap())
    }

    pub fn error(err: RpcError) -> Self {
        ResponseResult::Error(err)
    }
}

impl From<RpcError> for ResponseResult {
    fn from(err: RpcError) -> Self {
        ResponseResult::Error(err)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RpcResponse {
    jsonrpc: Version,

    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Id>,

    #[serde(flatten)]
    result: ResponseResult,
}

impl RpcResponse {
    pub fn new(id: Id, content: impl Into<ResponseResult>) -> Self {
        RpcResponse {
            jsonrpc: Version::V2,
            id: Some(id),
            result: content.into(),
        }
    }

    pub fn invalid_request(id: Id) -> Self {
        Self::new(id, RpcError::invalid_request())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Response {
    Single(RpcResponse),
    Batch(Vec<RpcResponse>),
}

impl Response {
    pub fn error(err: RpcError) -> Self {
        RpcResponse::new(Id::Null, err).into()
    }
}

impl From<RpcError> for Response {
    fn from(value: RpcError) -> Self {
        Response::error(value)
    }
}

impl From<RpcResponse> for Response {
    fn from(value: RpcResponse) -> Self {
        Response::Single(value)
    }
}
