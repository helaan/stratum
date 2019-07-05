use crate::template::TemplateContext;
use crate::AppState;
use actix_web::{error, http::Method, AsyncResponder, HttpRequest, Responder, Scope};
use askama::Template;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use futures::future::Future;
use stratum_db::models::Contest;
use stratum_db::schema::contests;
use stratum_db::Execute;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.route("", Method::GET, index)
}

#[derive(Template)]
#[template(path = "overview/index.html")]
struct IndexTemplate {
    ctx: TemplateContext,
    contests: Vec<Contest>,
}

pub fn index(req: HttpRequest<AppState>) -> impl Responder {
    req.state()
        .db
        .send(Execute::new(move |conn| {
            contests::dsl::contests
                .filter(contests::start_at.is_not_null())
                .load::<Contest>(&conn)
                .map_err(error::ErrorInternalServerError)
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok(contests) => Ok(IndexTemplate {
                ctx: TemplateContext::new(&req),
                contests,
            }),
            Err(e) => Err(e),
        })
        .responder()
}
