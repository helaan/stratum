use crate::multipart::parse_multipart;
use crate::util::render;
use crate::AppState;
use actix_web::{
    error, http::Method, AsyncResponder, Error, HttpMessage, HttpRequest, Responder, Scope,
};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use futures::future::Future;
use std::collections::HashMap;
use stratum_db::models::{Contest, Judgement, Problem, Submission, Team};
use stratum_db::schema::{contest_problems, judgements, problems, submission_files, submissions};
use stratum_db::Execute;
use tera::Context;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.route("", Method::GET, index)
        .route("/new", Method::POST, create)
}

pub fn index(req: HttpRequest<AppState>) -> impl Responder {
    let contest = req
        .extensions()
        .get::<Contest>()
        .map(|c| c.id)
        .ok_or_else(|| error::ErrorInternalServerError("contest not bound"));
    let team = req.extensions().get::<Team>().map(|t| t.id).ok_or_else(|| {
        error::ErrorForbidden("You are not allowed to submit as you are not in a team")
    });
    req.state()
        .db
        .send(Execute::new(|conn| {
            let contest_id = contest?;
            let team_id = team?;
            let problems = contest_problems::table
                .filter(contest_problems::contest_id.eq(contest_id))
                .inner_join(problems::table)
                .select(problems::all_columns)
                .load::<Problem>(&conn)
                .map_err(error::ErrorInternalServerError)?;
            let subs = Submission::belonging_to(&problems)
                .filter(submissions::team_id.eq(team_id))
                .left_join(
                    judgements::table.on(submissions::location_id
                        .eq(judgements::submission_location_id)
                        .and(submissions::id.eq(judgements::submission_id))
                        .and(judgements::valid.eq(true))),
                )
                .order(submissions::created_at.desc())
                .load::<(Submission, Option<Judgement>)>(&conn)
                .map_err(error::ErrorInternalServerError)?;

            Ok((problems, subs))
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok((problems, subs)) => {
                let mut ctx = Context::new();
                let hashed_problems: HashMap<_, _> = problems.iter().map(|v| (v.id, v)).collect();
                ctx.insert("problems", &problems);
                ctx.insert("hproblems", &hashed_problems);
                ctx.insert("submissions", &subs);
                render(&req, "contest/submission/index.html", ctx)
            }
            Err(e) => Err(e),
        })
        .responder()
}

pub fn create(req: HttpRequest<AppState>) -> impl Responder {
    let team = req.extensions().get::<Team>().map(|t| t.id).ok_or_else(|| {
        error::ErrorForbidden("You are not allowed to submit as you are not in a team")
    });
    parse_multipart(req.multipart())
        .then(move |x| {
            req.state().db.send(Execute::new(
                move |conn: PooledConnection<ConnectionManager<PgConnection>>| -> Result<_, Error> {
                    let team_id = team?;
                    let mut submission_cnt = 0;
                    let form = x?;
                    let source = form.get("source_code")?;
                    for (key, val) in form.content.iter() {
                        if key != "source_code" && !val.2.is_empty() {
                            let problem_id = key
                                .get(12..)
                                .ok_or_else(|| {
                                    error::ErrorBadRequest(format!(
                                        "Form field not matching pattern: {}",
                                        key
                                    ))
                                })?
                                .parse::<i64>()
                                .map_err(error::ErrorBadRequest)?;
                            let filename = val.1.get_filename().unwrap_or("textfile");
                            conn.transaction::<(), diesel::result::Error, _>(|| {
                                let sub = diesel::insert_into(submissions::table)
                                    .values((
                                        submissions::location_id.eq(1), //TODO
                                        submissions::team_id.eq(team_id),
                                        submissions::problem_id.eq(problem_id),
                                        submissions::entry_point.eq(filename),
                                    ))
                                    .get_results::<Submission>(&conn)?;
                                diesel::insert_into(submission_files::table)
                                    .values((
                                        submission_files::submission_id.eq(sub[0].id),
                                        submission_files::submission_location_id
                                            .eq(sub[0].location_id),
                                        submission_files::filename.eq(filename),
                                        submission_files::mimetype.eq(&val.0),
                                        submission_files::content.eq(val.2.as_ref()),
                                    ))
                                    .execute(&conn)?;
                                if submission_cnt == 0 {
                                    diesel::insert_into(submission_files::table)
                                        .values((
                                            submission_files::submission_id.eq(sub[0].id),
                                            submission_files::submission_location_id
                                                .eq(sub[0].location_id),
                                            submission_files::filename
                                                .eq(source.1.get_filename().unwrap_or("source")),
                                            submission_files::mimetype.eq(&source.0),
                                            submission_files::content.eq(source.2.as_ref()),
                                        ))
                                        .execute(&conn)?;
                                }
                                submission_cnt += 1;
                                Ok(())
                            })
                            .map_err(error::ErrorInternalServerError)?;
                        }
                    }
                    Ok(submission_cnt)
                },
            ))
        })
        .from_err()
        .and_then(|res| match res {
            Ok(n) => Ok(format!("uploaded {} submissions", n)),
            Err(e) => Err(e),
        })
        .responder()
}
