use actix_web::{Scope,HttpRequest,Responder};
use actix_web::http::Method;
use tera::Context;
use crate::util::render;
use crate::AppState;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.route("", Method::GET, index)
}

pub fn index(req: HttpRequest<AppState>) -> impl Responder {
    let ctx = Context::new();
    render(&req, "admin/index.html", ctx)
}
