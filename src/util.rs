use crate::models::{Team, User};
use crate::AppState;
use actix_web::{error, HttpRequest, HttpResponse};

// Render a page
// Arguments:
//  - state: AppState
//  - tpl: Path to template string
//  - ctx: Template context
pub fn render(
    state: &AppState,
    tpl: &str,
    ctx: &tera::Context,
) -> Result<HttpResponse, actix_web::Error> {
    let s = state
        .template
        .render(tpl, &ctx)
        .map_err(|e| error::ErrorInternalServerError(e.description().to_owned()))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

pub fn add_user_context(req: &HttpRequest<AppState>, ctx: &mut tera::Context) {
    ctx.insert("user", &req.extensions().get::<User>());
    ctx.insert("team", &req.extensions().get::<Team>());
}
