use crate::multipart::parse_multipart;
use crate::util::render;
use crate::AppState;
use actix_web::{
    error, http::Method, AsyncResponder, Error, HttpMessage, HttpRequest, HttpResponse, Path,
    Responder, Scope,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use futures::future::Future;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use stratum_db::models::{Contest, Judgement, Problem, Submission, Team, TestCaseJudgement};
use stratum_db::schema::{
    contest_problems, judgements, problems, submission_files, submissions, test_case_judgements,
};
use stratum_db::Execute;
use tera::Context;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.route("", Method::GET, index)
        .route("/new", Method::POST, create)
        .route("/{location_id:\\d+}/{id:\\d+}", Method::GET, show)
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
                // HACK: timestamptz too precise to notice
                //.distinct_on((submissions::location_id, submissions::id))
                .distinct_on(submissions::created_at)
                .filter(submissions::team_id.eq(team_id))
                .left_join(
                    judgements::table.on(submissions::location_id
                        .eq(judgements::submission_location_id)
                        .and(submissions::id.eq(judgements::submission_id))
                        .and(judgements::valid.eq(true))),
                )
                .order((submissions::created_at.desc(), judgements::grader_id.asc()))
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
    let location_id = req.state().location_id;
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
                                        submissions::location_id.eq(location_id),
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
            Ok(n) => Ok(HttpResponse::Ok().body(format!(
                r#"
<!DOCTYPE html>
<html lang="en">
<head>
	<meta charset="UTF-8">
	<title></title>
</head>
<body>
	<h1>Submitted {} submissions</h1>

	<p>We will grade them as soon as possible</p>

	<a href="javascript:history.back()">Go back to submission page</a>
</body>
</html>
                "#,
                n,
            ))),
            Err(e) => Err(e),
        })
        .responder()
}

#[derive(Deserialize)]
pub struct IdLocationIdParams {
    id: i64,
    location_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct Utf8TestCaseJudgement {
    pub judgement_id: i64,
    pub judgement_grader_id: i32,
    pub test_case_position: i32,
    pub status_code: i32,
    pub output: String,
    pub error: String,
    pub created_at: DateTime<Utc>,
}

fn show(req: HttpRequest<AppState>, params: Path<IdLocationIdParams>) -> impl Responder {
    let team = req
        .extensions()
        .get::<Team>()
        .ok_or_else(|| {
            error::ErrorForbidden("You are not allowed to submit as you are not in a team")
        })
        .map(|t| t.id);
    req.state()
        .db
        .send(Execute::new(move |conn| -> Result<_, Error> {
            let sub = submissions::table
                .find((params.location_id, params.id))
                .inner_join(problems::table)
                .first::<(Submission, Problem)>(&conn)
                .map_err(error::ErrorInternalServerError)?;
            let team_id = team?;
            if sub.0.team_id != team_id {
                return Err(error::ErrorNotFound("Could not find this submission"));
            }
            let jm = judgements::table
                .filter(judgements::submission_id.eq(params.id))
                .filter(judgements::submission_location_id.eq(params.location_id))
                .filter(judgements::valid.eq(true))
                .order(judgements::grader_id.asc())
                .first::<Judgement>(&conn)
                .map_err(error::ErrorInternalServerError)?;
            let tcjs = test_case_judgements::table
                .filter(test_case_judgements::judgement_id.eq(jm.id))
                .filter(test_case_judgements::judgement_grader_id.eq(jm.grader_id))
                .order(test_case_judgements::test_case_position.asc())
                .load::<TestCaseJudgement>(&conn)
                .map(|v| {
                    v.into_iter()
                        .map(|tcj| Utf8TestCaseJudgement {
                            judgement_id: tcj.judgement_id,
                            judgement_grader_id: tcj.judgement_grader_id,
                            test_case_position: tcj.test_case_position,
                            status_code: tcj.status_code,
                            output: String::from_utf8(tcj.output).unwrap(),
                            error: String::from_utf8(tcj.error).unwrap(), //HACK
                            created_at: tcj.created_at,
                        })
                        .collect::<Vec<_>>()
                })
                .map_err(error::ErrorInternalServerError)?;
            Ok((sub, jm, tcjs))
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok((sub, jm, tcjs)) => {
                let mut ctx = Context::new();
                ctx.insert("submission", &sub.0);
                ctx.insert("problem", &sub.1);
                ctx.insert("judgement", &jm);
                ctx.insert("test_case_judgements", &tcjs);
                render(&req, "contest/submission/show.html", ctx)
            }
            Err(e) => Err(e),
        })
        .responder()
}
