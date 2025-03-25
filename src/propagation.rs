use tracing_opentelemetry::OpenTelemetrySpanExt;

struct ReqwestHeaderMapWrapper<'a>(&'a mut reqwest::header::HeaderMap);

impl opentelemetry::propagation::Injector for ReqwestHeaderMapWrapper<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.0.append(
            reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
            reqwest::header::HeaderValue::from_bytes(value.as_bytes()).unwrap(),
        );
    }
}

pub fn injecte_into_header_map() -> reqwest::header::HeaderMap {
    let span = tracing::Span::current();
    let mut header_map = reqwest::header::HeaderMap::new();
    opentelemetry::global::get_text_map_propagator(|propagator| {
        let mut wrapper = ReqwestHeaderMapWrapper(&mut header_map);
        propagator.inject_context(&span.context(), &mut wrapper);
    });
    header_map
}

struct RocketHeaderMapWrapper<'a>(&'a rocket::http::HeaderMap<'a>);

impl opentelemetry::propagation::Extractor for RocketHeaderMapWrapper<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get_one(key)
    }

    fn keys(&self) -> Vec<&str> {
        unimplemented!()
    }
}

pub fn extract_from_header_map(header_map: &rocket::http::HeaderMap, span: &mut tracing::Span) {
    let wrapper = RocketHeaderMapWrapper(header_map);
    let parent_context =
        opentelemetry::global::get_text_map_propagator(|propagator| propagator.extract(&wrapper));
    span.set_parent(parent_context);
}
