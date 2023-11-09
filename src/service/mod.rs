pub mod code;
pub mod email;
pub mod error;
pub mod serde_helpers;
pub mod verify;

use ethers::providers::{Http, Provider};
use sqlx::PgPool;
use tracing::trace;

use self::error::ToRpcResponseResult;
use crate::{rpc::response::ResponseResult, server::handler::RpcHandler};

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum ApiRequest {
    #[serde(rename = "send_code")]
    SendCode(String, String),
    #[serde(rename = "verify_code")]
    VerifyCode(String, String, String),
}

#[derive(Clone)]
pub struct Context {
    pub db: PgPool,
    pub provider: Provider<Http>,
    pub guardian_address: String,
    pub signer: String,
}

#[derive(Clone)]
pub struct HttpRpcHandler {
    context: Context,
}

impl HttpRpcHandler {
    pub fn new(context: Context) -> Self {
        HttpRpcHandler { context }
    }

    pub async fn execute(&self, request: ApiRequest) -> ResponseResult {
        trace!(target: "rpc::api", "executing eth request");
        match request {
            ApiRequest::SendCode(account, email) => {
                code::generate_code(&self.context.db, account, email)
                    .await
                    .to_rpc_result()
            }
            ApiRequest::VerifyCode(account, email, code) => {
                verify::verify_code(&self.context, account, email, code)
                    .await
                    .to_rpc_result()
            }
        }
    }
}

#[async_trait::async_trait]
impl RpcHandler for HttpRpcHandler {
    type Request = ApiRequest;

    async fn on_request(&self, request: Self::Request) -> ResponseResult {
        self.execute(request).await
    }
}
