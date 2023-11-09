use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Version {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum RequestParams {
    None,
    Array(Vec<serde_json::Value>),
    Object(serde_json::Map<String, serde_json::Value>),
}

impl From<RequestParams> for serde_json::Value {
    fn from(params: RequestParams) -> Self {
        match params {
            RequestParams::None => serde_json::Value::Null,
            RequestParams::Array(arr) => arr.into(),
            RequestParams::Object(obj) => obj.into(),
        }
    }
}

fn no_params() -> RequestParams {
    RequestParams::None
}

fn null_id() -> Id {
    Id::Null
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Id {
    String(String),
    Number(i64),
    Null,
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Id::String(s) => s.fmt(f),
            Id::Number(n) => n.fmt(f),
            Id::Null => f.write_str("null"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RpcMethodCall {
    pub jsonrpc: Version,
    pub method: String,
    #[serde(default = "no_params")]
    pub params: RequestParams,
    pub id: Id,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RpcNotification {
    pub jsonrpc: Option<Version>,
    pub method: String,
    #[serde(default = "no_params")]
    pub params: RequestParams,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RpcCall {
    MethodCall(RpcMethodCall),
    Notification(RpcNotification),
    Invalid {
        #[serde(default = "null_id")]
        id: Id,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, untagged)]
pub enum Request {
    Single(RpcCall),
    Batch(Vec<RpcCall>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_serialize_batch() {
        let batch = Request::Batch(vec![
            RpcCall::MethodCall(RpcMethodCall {
                jsonrpc: Version::V2,
                method: "test1".to_owned(),
                params: RequestParams::Array(vec![
                    serde_json::Value::from("hello"),
                    serde_json::Value::from("rust"),
                ]),
                id: Id::Number(1),
            }),
            RpcCall::MethodCall(RpcMethodCall {
                jsonrpc: Version::V2,
                method: "test2".to_owned(),
                params: RequestParams::Array(vec![serde_json::Value::from(123)]),
                id: Id::Number(2),
            }),
        ]);

        let obj = serde_json::to_string(&batch).unwrap();
        assert_eq!(
            obj,
            r#"[{"jsonrpc":"2.0","method":"test1","params":["hello","rust"],"id":1},{"jsonrpc":"2.0","method":"test2","params":[123],"id":2}]"#,
        )
    }

    #[test]
    fn can_deserialize_batch() {
        let s = r#"[{"jsonrpc":"2.0","method":"test1","params":["hello","rust"],"id":1},{"jsonrpc":"2.0","method":"test2","params":[123],"id":2}]"#;
        let obj: Request = serde_json::from_str(s).unwrap();

        assert_eq!(
            obj,
            Request::Batch(vec![
                RpcCall::MethodCall(RpcMethodCall {
                    jsonrpc: Version::V2,
                    method: "test1".to_owned(),
                    params: RequestParams::Array(vec![
                        serde_json::Value::from("hello"),
                        serde_json::Value::from("rust"),
                    ]),
                    id: Id::Number(1),
                }),
                RpcCall::MethodCall(RpcMethodCall {
                    jsonrpc: Version::V2,
                    method: "test2".to_owned(),
                    params: RequestParams::Array(vec![serde_json::Value::from(123)]),
                    id: Id::Number(2),
                }),
            ]),
        );
    }
}
