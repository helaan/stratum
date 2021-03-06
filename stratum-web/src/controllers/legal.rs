use crate::template::TemplateContext;
use crate::AppState;
use actix_web::{http::Method, HttpRequest, HttpResponse, Responder, Scope};
use askama::Template;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.route("", Method::GET, index)
        .route("download", Method::GET, download_source)
}

#[derive(Template)]
#[template(path = "legal/index.html")]
struct IndexTemplate {
    ctx: TemplateContext,
}

fn index(req: HttpRequest<AppState>) -> impl Responder {
    IndexTemplate {
        ctx: TemplateContext::new(&req),
    }
}

include!(concat!(env!("OUT_DIR"), "/embed_source.rs"));
