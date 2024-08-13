[Docs](https://docs.rs/http_metware/0.1.1/http_metware/)

This tower middleware can be usefull to easily integrate http metrics into your web server.
It doesnt rely on any metrics lib, all you have to do is to access `ExposedMetrics` via `MetricsExposer` trait implemetation and log them or collect via any metrics lib like prometheus client.
```rust
pub struct ExposedMetrics<'a> {
    pub uri: &'a Uri,
    pub method: &'a str,
    pub status_code: u16,
    pub elapsed_time: &'a Duration,
}
```
