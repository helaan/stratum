use crate::database::Execute;
use crate::models::{Contest, ContestProblem, Problem, Team};
use crate::schema::{contestproblems, problems, teams};
use crate::util::render;
use crate::AppState;
use actix_web::{error, http::Method, AsyncResponder, Error, HttpRequest, HttpResponse, Scope};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use futures::future::Future;
use tera::Context;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.route("", Method::GET, index)
}

pub fn index(
    req: HttpRequest<AppState>,
) -> Result<Box<(Future<Item = HttpResponse, Error = Error>)>, Error> {
    let contest_id = req
        .extensions()
        .get::<Contest>()
        .map(|c| c.id)
        .ok_or(error::ErrorInternalServerError("contest not bound"))?;
    Ok(req
        .state()
        .db
        .send(Execute::new(move |s| -> Result<_, Error> {
            let conn = s.get_conn()?;
            let cproblems = contestproblems::table
                .filter(contestproblems::contest_id.eq(contest_id))
                .inner_join(problems::table)
                .load::<(ContestProblem, Problem)>(&conn)
                .map_err(|e| error::ErrorInternalServerError(e))?;
            let teams = teams::table
                .load::<Team>(&conn)
                .map_err(|e| error::ErrorInternalServerError(e))?;
            Ok((cproblems, teams))
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok((cproblems, teams)) => {
                let mut ctx = Context::new();
                ctx.insert("cproblems", &cproblems);
                ctx.insert("teams", &teams);
                render(&req, "contest/scoreboard/show.html", ctx)
            }
            Err(e) => Err(e),
        })
        .responder())
}
