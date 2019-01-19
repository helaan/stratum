use crate::database::Execute;
use crate::models::Contest;
use crate::schema::contests;
use crate::util::render;
use crate::AppState;
use actix_web::{error, http::Method, AsyncResponder, Error, HttpRequest, Responder, Scope};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use futures::future::Future;
use tera::Context;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.route("", Method::GET, index)
}

pub fn index(req: HttpRequest<AppState>) -> impl Responder {
    req.state()
        .db
        .send(Execute::new(move |s| -> Result<Vec<Contest>, Error> {
            let conn = s.get_conn()?;
            contests::dsl::contests
                .filter(contests::start_at.is_not_null())
                .load(&conn)
                .map_err(|e| error::ErrorInternalServerError(e))
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok(contests) => {
                let mut ctx = Context::new();
                ctx.insert("contests", &contests);
                render(&req, "overview/index.html", ctx)
            }
            Err(e) => Err(e),
        })
        .responder()
}
