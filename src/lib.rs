use axum::response::Response as HttpResponse;
use std::task::{Context, Poll};
use std::time::Duration;

use axum::extract::Request as HttpRequest;
use axum::http::Method;
use http::Uri;
use tower::Service;

use future::HttpMetricsFuture;

pub use layer::HttpMetricsLayer;

mod future;
mod layer;

pub struct ExposedMetrics<'a> {
    pub uri: &'a Uri,
    pub method: &'a str,
    pub status_code: u16,
    pub elapsed_time: &'a Duration,
}

/// This trait is used to implement custom logic for [`HttpMetrics`]
/// Method intercerp_response can be used for collecting custom metrics
/// Method expose is used for exposing all collected metrics (default metrics and custom)
pub trait MetricsExposer {
    fn intercerp_response(&mut self, _resp: &HttpResponse) {}

    fn intercerp_request(&mut self, _req: &HttpRequest) {}

    fn expose(&self, metrics: ExposedMetrics);
}

/// This middleware collects default metrics [`ExposedMetrics`] and then exposes them via [`MetricsExposer`] trait
#[derive(Clone)]
pub struct HttpMetrics<S, ME> {
    service: S,
    exposer: ME,
}

impl<S, ME> HttpMetrics<S, ME>
where
    ME: MetricsExposer + Clone,
{
    pub fn new(service: S, exposer: ME) -> Self {
        Self { service, exposer }
    }
}

impl<S, ME> Service<HttpRequest> for HttpMetrics<S, ME>
where
    S: Service<HttpRequest, Response = HttpResponse> + Clone,
    ME: MetricsExposer + Clone,
{
    type Response = HttpResponse;
    type Error = S::Error;
    type Future = HttpMetricsFuture<S::Future, ME>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        let method_label = label_from_method(request.method());
        let uri = request.uri().clone();

        self.exposer.intercerp_request(&request);

        HttpMetricsFuture::new(
            self.service.call(request),
            method_label,
            uri,
            self.exposer.clone(),
        )
    }
}

/// this function is needed because we need to know every method name str at compile time
fn label_from_method(method: &Method) -> &'static str {
    match *method {
        Method::CONNECT => "CONNECT",
        Method::DELETE => "DELETE",
        Method::GET => "GET",
        Method::HEAD => "HEAD",
        Method::OPTIONS => "OPTIONS",
        Method::PATCH => "PATCH",
        Method::POST => "POST",
        Method::PUT => "PUT",
        Method::TRACE => "TRACE",
        _ => "unknown",
    }
}
