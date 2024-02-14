use std::{fmt, net::SocketAddr};

use axum::{
    extract::{rejection::JsonRejection, Extension},
    routing::{post, IntoMakeService},
    Json, Router, Server,
};
use futures::{future, FutureExt};
use hyper::{server::conn::AddrIncoming, Method};
use serde::de::DeserializeOwned;
use tower_http::{
    cors::{AllowHeaders, AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use tracing::{error, trace, warn};

use crate::rpc::{
    error::RpcError,
    request::{Request, RpcCall, RpcMethodCall},
    response::{Response, ResponseResult, RpcResponse},
};

pub type RpcServer = Server<AddrIncoming, IntoMakeService<Router>>;

#[async_trait::async_trait]
pub trait RpcHandler: Clone + Send + Sync + 'static {
    type Request: DeserializeOwned + Send + Sync + fmt::Debug;

    async fn on_request(&self, request: Self::Request) -> ResponseResult;

    async fn on_call(&self, call: RpcMethodCall) -> RpcResponse {
        trace!(target: "rpc", id = ?call.id, method = ?call.method, "received method call");

        let RpcMethodCall {
            method, params, id, ..
        } = call;

        let params: serde_json::Value = params.into();
        let call = serde_json::json!({
            "method": &method,
            "params": params
        });

        match serde_json::from_value::<Self::Request>(call) {
            Ok(req) => {
                let result = self.on_request(req).await;
                RpcResponse::new(id, result)
            }
            Err(err) => {
                let err = err.to_string();
                if err.contains("unknow variant") {
                    error!(target: "rpc", ?method, "failed to deserialize method due to unknow variant");
                    RpcResponse::new(id, RpcError::method_not_found())
                } else {
                    error!(target: "rpc", ?method, ?err, "failed to deserialize method");
                    RpcResponse::new(id, RpcError::invalid_params(err))
                }
            }
        }
    }
}

pub async fn handle<Handler: RpcHandler>(
    request: Result<Json<Request>, JsonRejection>,
    Extension(handler): Extension<Handler>,
) -> Json<Response> {
    match request {
        Err(err) => {
            warn!(target: "rpc", ?err, "invalid request");
            Response::error(RpcError::invalid_request()).into()
        }
        Ok(req) => handle_request(req.0, handler)
            .await
            .unwrap_or_else(|| Response::error(RpcError::invalid_request()))
            .into(),
    }
}

pub async fn handle_request<Handler: RpcHandler>(
    req: Request,
    handler: Handler,
) -> Option<Response> {
    fn responses_as_batch(outs: Vec<Option<RpcResponse>>) -> Option<Response> {
        let batch: Vec<_> = outs.into_iter().flatten().collect();
        (!batch.is_empty()).then_some(Response::Batch(batch))
    }

    match req {
        Request::Single(call) => handle_call(call, handler).await.map(Response::Single),
        Request::Batch(calls) => {
            future::join_all(
                calls
                    .into_iter()
                    .map(move |call| handle_call(call, handler.clone())),
            )
            .map(responses_as_batch)
            .await
        }
    }
}

async fn handle_call<Handler: RpcHandler>(call: RpcCall, handler: Handler) -> Option<RpcResponse> {
    match call {
        RpcCall::MethodCall(call) => {
            trace!(target: "rpc", id = ?call.id , method = ?call.method,  "handling call");
            Some(handler.on_call(call).await)
        }
        RpcCall::Notification(notification) => {
            trace!(target: "rpc", method = ?notification.method, "received rpc notification");
            None
        }
        RpcCall::Invalid { id } => {
            warn!(target: "rpc", ?id,  "invalid rpc call");
            Some(RpcResponse::invalid_request(id))
        }
    }
}

pub fn serve_http<Http>(addr: SocketAddr, http: Http) -> RpcServer
where
    Http: RpcHandler,
{
    let svc = Router::new()
        .route("/", post(handle::<Http>))
        .layer(Extension(http))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::any())
                .allow_headers(AllowHeaders::any())
                .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS]),
        )
        .into_make_service();

    Server::bind(&addr).serve(svc)
}
