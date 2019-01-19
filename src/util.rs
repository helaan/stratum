use crate::models::{Contest, Team, User};
use crate::AppState;
use actix_web::{error, Error, HttpRequest, HttpResponse};

// Render a page
// Arguments:
//  - state: AppState
//  - tpl: Path to template string
//  - ctx: Template context
pub fn render(
    req: &HttpRequest<AppState>,
    tpl: &str,
    ctx: tera::Context,
) -> Result<HttpResponse, Error> {
    let body = render_string(req, tpl, ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

pub fn render_string(
    req: &HttpRequest<AppState>,
    tpl: &str,
    mut ctx: tera::Context,
) -> Result<String, Error> {
    ctx.insert("active_user", &req.extensions().get::<User>());
    ctx.insert("active_team", &req.extensions().get::<Team>());
    ctx.insert("app_version", env!("CARGO_PKG_VERSION"));
    if let Some(contest) = &req.extensions().get::<Contest>() {
        ctx.insert("contest", contest);
    }
    req.state()
        .template
        .render(tpl, &ctx)
        .map_err(|e| error::ErrorInternalServerError(e.description().to_owned()))
}
