use crate::util::render;
use crate::AppState;
use actix_web::http::Method;
use actix_web::{HttpRequest, Responder, Scope};
use tera::Context;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.route("", Method::GET, index)
}

pub fn index(req: HttpRequest<AppState>) -> impl Responder {
    let ctx = Context::new();
    render(&req, "admin/index.html", ctx)
}
