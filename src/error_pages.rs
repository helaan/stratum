use crate::util::render_string;
use crate::AppState;
use actix_web::middleware::{ErrorHandlers, Response};
use actix_web::{http, Error, HttpRequest, HttpResponse};
use tera::Context;

pub fn register() -> ErrorHandlers<AppState> {
    ErrorHandlers::new()
        .handler(http::StatusCode::NOT_FOUND, render_404)
        .handler(http::StatusCode::INTERNAL_SERVER_ERROR, render_500)
}

pub fn render_404(req: &HttpRequest<AppState>, resp: HttpResponse) -> Result<Response, Error> {
    render_error("error_pages/404.html", req, resp)
}

pub fn render_500(req: &HttpRequest<AppState>, resp: HttpResponse) -> Result<Response, Error> {
    render_error("error_pages/500.html", req, resp)
}

pub fn render_error(
    tpl: &str,
    req: &HttpRequest<AppState>,
    resp: HttpResponse,
) -> Result<Response, Error> {
    let mut builder = resp.into_builder();
    let ctx = Context::new();
    let body = render_string(req, tpl, ctx)?;
    let new_resp = builder.body(body);
    Ok(Response::Done(new_resp))
}
