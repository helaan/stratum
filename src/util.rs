use crate::models::{Team, User};
use crate::AppState;
use actix_web::{error, HttpRequest, HttpResponse};

// Render a page
// Arguments:
//  - state: AppState
//  - tpl: Path to template string
//  - ctx: Template context
pub fn render(
    req: &HttpRequest<AppState>,
    tpl: &str,
    mut ctx: tera::Context,
) -> Result<HttpResponse, actix_web::Error> {
    ctx.insert("active_user", &req.extensions().get::<User>());
    ctx.insert("active_team", &req.extensions().get::<Team>());
    ctx.insert("app_version", env!("CARGO_PKG_VERSION"));
    let s = req.state()
        .template
        .render(tpl, &ctx)
        .map_err(|e| error::ErrorInternalServerError(e.description().to_owned()))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
