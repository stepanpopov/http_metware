use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};

use axum::response::Response as HttpResponse;
use futures::ready;
use http::Uri;
use pin_project_lite::pin_project;

use super::{ExposedMetrics, MetricsExposer};

pin_project! {
    pub struct HttpMetricsFuture<Fut, ME> {
        #[pin]
        inner: Fut,
        method: &'static str,
        uri: Uri,
        start: Instant,
        exposer: ME,
    }
}

impl<Fut, ME> HttpMetricsFuture<Fut, ME> {
    pub(crate) fn new(inner: Fut, method: &'static str, uri: Uri, exposer: ME) -> Self {
        Self {
            inner,
            method,
            uri,
            start: Instant::now(),
            exposer,
        }
    }
}

impl<Fut, ME, E> Future for HttpMetricsFuture<Fut, ME>
where
    Fut: Future<Output = Result<HttpResponse, E>>,
    ME: MetricsExposer + Clone,
{
    type Output = Result<HttpResponse, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let method = self.method;
        let uri = self.uri.clone();
        let elapsed_time = self.start.elapsed();

        let mut exposer = self.exposer.clone();

        let this = self.project();

        match ready!(this.inner.poll(cx)) {
            Ok(res) => {
                exposer.intercerp_response(&res);

                let status_code = res.status().as_u16();
                exposer.expose(ExposedMetrics {
                    uri: &uri,
                    method,
                    status_code,
                    elapsed_time: &elapsed_time,
                });
                Poll::Ready(Ok(res))
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}
