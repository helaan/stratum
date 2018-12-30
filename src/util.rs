use actix_web::{error, HttpResponse};
use crate::AppState;

// Render a page
// Arguments:
//  - state: AppState
//  - tpl: Path to template string
//  - ctx: Template context
pub fn render(state: &AppState, tpl: &str, ctx: &tera::Context)
        -> Result<HttpResponse, actix_web::Error> {
    let s = state.template.render(tpl, &ctx)
        .map_err(|e| error::ErrorInternalServerError(e.description().to_owned()))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
