use std::fmt::Debug;

use tower_http::{
    classify::GrpcFailureClass,
    trace::{OnFailure, OnRequest, OnResponse},
};

#[derive(Debug, Clone, Default)]
pub struct LogOnRequest {}

impl<B> OnRequest<B> for LogOnRequest {
    fn on_request(
        &mut self,
        request: &http::Request<B>,
        _span: &tracing::Span,
    ) {
        tracing::log::info!(
            target: "grpc-request",
            "{:?} {} {} {:?}",
            request.version(),
            request.method(),
            request.uri(),
            request.headers()
        );
    }
}

#[derive(Debug, Clone, Default)]
pub struct LogOnResponse {}

impl<B> OnResponse<B> for LogOnResponse {
    fn on_response(
        self,
        response: &tonic::codegen::http::Response<B>,
        _latency: std::time::Duration,
        _span: &tracing::Span,
    ) {
        tracing::log::info!(
            target: "grpc-response",
            "{:?} {} {:?}",
            response.version(),
            response.status(),
            response.headers(),
        );
    }
}

#[derive(Debug, Clone, Default)]
pub struct LogOnFailure {}

impl OnFailure<GrpcFailureClass> for LogOnFailure {
    fn on_failure(
        &mut self,
        failure_classification: GrpcFailureClass,
        _latency: std::time::Duration,
        _span: &tracing::Span,
    ) {
        tracing::log::info!(
            target: "grpc-failure",
            "{}",
            failure_classification
        );
    }
}
