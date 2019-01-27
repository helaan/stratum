use crate::util::render;
use crate::AppState;
use actix_web::{
    error, http::Method, AsyncResponder, Error, HttpRequest, HttpResponse, Path, Responder, Scope,
};
use diesel::prelude::*;
use diesel::BelongingToDsl;
use futures::future::Future;
use serde::{Deserialize, Serialize};
use stratum_db::models::{Contest, ContestProblem, Problem, ProblemStatement};
use stratum_db::schema::{contest_problems, problem_statements, problems};
use stratum_db::Execute;
use tera::Context;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.route("", Method::GET, index)
        .route("/{id}", Method::GET, show)
}

#[derive(Identifiable, Associations, Queryable, Serialize, Deserialize, Debug)]
#[belongs_to(Problem)]
#[table_name = "problem_statements"]
struct LightProblemStatement {
    id: i64,
    problem_id: i64,
    filename: String,
    mime_type: String,
}

pub fn index(req: HttpRequest<AppState>) -> impl Responder {
    let contest_id = req
        .extensions()
        .get::<Contest>()
        .map(|c| c.id)
        .ok_or_else(|| error::ErrorInternalServerError("contest not bound"));
    req.state()
        .db
        .send(Execute::new(|conn| {
            let cid = contest_id?;
            let cp_problems = contest_problems::table
                .inner_join(problems::table)
                .filter(contest_problems::contest_id.eq(cid))
                .order(contest_problems::label.asc())
                .load::<(ContestProblem, Problem)>(&conn)
                .map_err(error::ErrorInternalServerError)?;
            let (cproblems, problems): (Vec<_>, Vec<_>) = cp_problems.into_iter().unzip();
            let problem_statements = ProblemStatement::belonging_to(&problems)
                .select((
                    problem_statements::id,
                    problem_statements::problem_id,
                    problem_statements::filename,
                    problem_statements::mimetype,
                ))
                .load::<LightProblemStatement>(&conn)
                .map_err(error::ErrorInternalServerError)?
                .grouped_by(&problems);
            Ok(cproblems
                .into_iter()
                .zip(problems)
                .zip(problem_statements)
                .collect::<Vec<((ContestProblem, Problem), Vec<LightProblemStatement>)>>())
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok(problems) => {
                let mut ctx = Context::new();
                ctx.insert("problems", &problems);
                render(&req, "contest/problem/index.html", ctx)
            }
            Err(e) => Err(e),
        })
        .responder()
}

#[derive(Deserialize)]
pub struct IdParams {
    id: i64,
}

pub fn show(req: HttpRequest<AppState>, params: Path<IdParams>) -> impl Responder {
    let contest_id = req.extensions().get::<Contest>().unwrap().id;
    req.state()
        .db
        .send(Execute::new(move |conn| {
            let statement = problem_statements::table
                .find(params.id)
                .get_result::<ProblemStatement>(&conn)
                .map_err(error::ErrorInternalServerError)?;
            contest_problems::table
                .filter(contest_problems::contest_id.eq(contest_id))
                .filter(contest_problems::problem_id.eq(statement.problem_id))
                .first::<ContestProblem>(&conn)
                .optional()
                .map_err(error::ErrorInternalServerError)?
                .ok_or_else(|| error::ErrorNotFound("Could not find ProblemStatement"))?;
            Ok(statement)
        }))
        .from_err()
        .and_then(|res: Result<ProblemStatement, Error>| match res {
            Ok(statement) => Ok(HttpResponse::Ok()
                .header("Content-Type", statement.mimetype)
                .header(
                    "Content-Disposition",
                    format!("attachment; filename=\"{}\"", statement.filename),
                )
                .body(statement.statement)),
            Err(e) => Err(e),
        })
        .responder()
}
