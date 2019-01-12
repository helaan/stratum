use crate::util::{add_user_context, render};
use crate::AppState;
use actix_web::{http::Method, HttpRequest, Responder, Scope};
use tera::Context;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.route("", Method::GET, show)
}

pub fn show(req: HttpRequest<AppState>) -> impl Responder {
    let mut ctx = Context::new();
    add_user_context(&req, &mut ctx);
    render(req.state(), "scoreboard/show.html", &ctx)
}
