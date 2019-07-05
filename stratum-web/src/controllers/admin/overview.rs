use crate::template::TemplateContext;
use crate::AppState;
use actix_web::http::Method;
use actix_web::{HttpRequest, Responder, Scope};
use askama::Template;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.route("", Method::GET, index)
}

#[derive(Template)]
#[template(path = "admin/index.html")]
struct IndexTemplate {
    ctx: TemplateContext,
}

pub fn index(req: HttpRequest<AppState>) -> impl Responder {
    IndexTemplate {
        ctx: TemplateContext::new(&req),
    }
}
