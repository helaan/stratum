use crate::util::render;
use crate::AppState;
use actix_web::{error, http::Method, AsyncResponder, HttpRequest, Path, Responder, Scope};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use futures::future::Future;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use stratum_db::models::{Judgement, Problem, Submission, Team, TestCaseJudgement};
use stratum_db::schema::{
    judgements, problems, submission_files, submissions, teams, test_case_judgements,
};
use stratum_db::Execute;
use tera::Context;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.route("", Method::GET, index)
        .route("/{location_id:\\d+}/{id:\\d+}", Method::GET, show)
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
    req.state()
        .db
        .send(Execute::new(move |conn| {
            let sub = submissions::table
                .find((params.location_id, params.id))
                .inner_join(teams::table)
                .inner_join(problems::table)
                .first::<(Submission, Team, Problem)>(&conn)
                .map_err(error::ErrorInternalServerError)?;
            let filenames = submission_files::table
                .select(submission_files::filename)
                .filter(submission_files::submission_id.eq(params.id))
                .filter(submission_files::submission_location_id.eq(params.location_id))
                .load::<String>(&conn)
                .map_err(error::ErrorInternalServerError)?;
            let jms = judgements::table
                .filter(judgements::submission_id.eq(params.id))
                .filter(judgements::submission_location_id.eq(params.location_id))
                .left_join(
                    test_case_judgements::table.on(judgements::id
                        .eq(test_case_judgements::judgement_id)
                        .and(judgements::grader_id.eq(test_case_judgements::judgement_grader_id))),
                )
                .order((judgements::grader_id.asc(), judgements::id.asc()))
                .load::<(Judgement, Option<TestCaseJudgement>)>(&conn)
                .map(|v| {
                    v.into_iter()
                        .map(|tup| {
                            (
                                tup.0,
                                tup.1.map(|tcj| Utf8TestCaseJudgement {
                                    judgement_id: tcj.judgement_id,
                                    judgement_grader_id: tcj.judgement_grader_id,
                                    test_case_position: tcj.test_case_position,
                                    status_code: tcj.status_code,
                                    output: String::from_utf8(tcj.output).unwrap(),
                                    error: String::from_utf8(tcj.error).unwrap(), //HACK
                                    created_at: tcj.created_at,
                                }),
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .map_err(error::ErrorInternalServerError)?;
            Ok((sub, filenames, jms))
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok((sub, filenames, jms)) => {
                let mut ctx = Context::new();
                ctx.insert("submission", &sub.0);
                ctx.insert("team", &sub.1);
                ctx.insert("problem", &sub.2);
                ctx.insert("filenames", &filenames);
                ctx.insert("judgements", &jms);
                render(&req, "admin/submission/show.html", ctx)
            }
            Err(e) => Err(e),
        })
        .responder()
}

fn index(req: HttpRequest<AppState>) -> impl Responder {
    req.state()
        .db
        .send(Execute::new(|conn| {
            let subs = submissions::table
                .order(submissions::created_at.asc())
                .load::<Submission>(&conn)
                .map_err(error::ErrorInternalServerError)?;
            let jms = judgements::table
                .load::<Judgement>(&conn)
                .map_err(error::ErrorInternalServerError)?
                .into_iter()
                .fold(HashMap::new(), |mut acc, item| {
                    let k = (item.submission_id, item.submission_location_id);
                    acc.entry(k.0)
                        .or_insert_with(HashMap::new)
                        .entry(k.1)
                        .or_insert_with(Vec::new);
                    acc.get_mut(&k.0).unwrap().get_mut(&k.1).unwrap().push(item);
                    acc
                });
            Ok((subs, jms))
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok((subs, jms)) => {
                let mut ctx = Context::new();
                ctx.insert("submissions", &subs);
                ctx.insert("judgements", &jms);
                render(&req, "admin/submission/index.html", ctx)
            }
            Err(e) => Err(e),
        })
        .responder()
}
