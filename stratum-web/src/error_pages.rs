use crate::template::TemplateContext;
use crate::AppState;
use actix_web::middleware::{ErrorHandlers, Response};
use actix_web::{error, http, Error, HttpRequest, HttpResponse};
use askama::Template;

pub fn register() -> ErrorHandlers<AppState> {
    ErrorHandlers::new()
        .handler(http::StatusCode::NOT_FOUND, render_404)
        .handler(http::StatusCode::INTERNAL_SERVER_ERROR, render_500)
}

#[derive(Template)]
#[template(path = "error_pages/404.html")]
struct Error404Template {
    ctx: TemplateContext,
}

#[derive(Template)]
#[template(path = "error_pages/500.html")]
struct Error500Template {
    ctx: TemplateContext,
}

fn render_404(req: &HttpRequest<AppState>, resp: HttpResponse) -> Result<Response, Error> {
    render_error(
        resp,
        Error404Template {
            ctx: TemplateContext::new(&req),
        },
    )
}

fn render_500(req: &HttpRequest<AppState>, resp: HttpResponse) -> Result<Response, Error> {
    render_error(
        resp,
        Error500Template {
            ctx: TemplateContext::new(&req),
        },
    )
}

pub fn render_error(resp: HttpResponse, tpl: impl Template) -> Result<Response, Error> {
    let result = tpl.render().map_err(error::ErrorInternalServerError)?;
    let mut builder = resp.into_builder();
    let new_resp = builder.body(result);
    Ok(Response::Done(new_resp))
}
