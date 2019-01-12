use crate::util::render;
use crate::AppState;
use actix_web::{http::Method, HttpRequest, Responder, Scope};
use tera::Context;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.route("", Method::GET, show)
}

pub fn show(req: HttpRequest<AppState>) -> impl Responder {
    let ctx = Context::new();
    render(&req, "scoreboard/show.html", ctx)
}
