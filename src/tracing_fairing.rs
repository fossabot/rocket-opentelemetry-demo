use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Status;
use rocket::request::FromRequest;
use rocket::request::Outcome;
use rocket::{Data, Request, Response};
use tracing::{info_span, Span};

#[derive(Clone)]
pub struct TracingSpan<T = Span>(pub T);

pub struct TracingFairing;

#[rocket::async_trait]
impl Fairing for TracingFairing {
    fn info(&self) -> Info {
        Info {
            name: "Tracing Fairing",
            kind: Kind::Request | Kind::Response,
        }
    }
    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        let mut span = info_span!(
            "request",
            otel.name=%req.uri().path(),
            http.method = %req.method(),
            http.status_code = tracing::field::Empty,
        );
        crate::extract_from_header_map(req.headers(), &mut span);

        req.local_cache(|| TracingSpan::<Option<Span>>(Some(span)));
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        if let Some(span) = req
            .local_cache(|| TracingSpan::<Option<Span>>(None))
            .0
            .to_owned()
        {
            let span = span.entered();
            span.record("http.status_code", res.status().code);
            drop(span);
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for TracingSpan {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, ()> {
        match request.local_cache(|| TracingSpan::<Option<Span>>(None)) {
            TracingSpan(Some(span)) => Outcome::Success(TracingSpan(span.to_owned())),
            TracingSpan(None) => Outcome::Error((Status::InternalServerError, ())),
        }
    }
}
