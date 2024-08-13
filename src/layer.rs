use tower::Layer;

use super::{HttpMetrics, MetricsExposer};

#[derive(Clone)]
pub struct HttpMetricsLayer<ME> {
    exposer: ME,
}

impl<ME> HttpMetricsLayer<ME> {
    pub fn new(exposer: ME) -> Self {
        Self { exposer }
    }
}

impl<S, ME> Layer<S> for HttpMetricsLayer<ME>
where
    ME: MetricsExposer + Clone,
{
    type Service = HttpMetrics<S, ME>;

    fn layer(&self, inner: S) -> Self::Service {
        HttpMetrics::new(inner, self.exposer.clone())
    }
}
