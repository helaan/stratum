use crate::util::render;
use crate::AppState;
use actix_web::{error, http::Method, AsyncResponder, Error, HttpRequest, HttpResponse, Scope};
use chrono::Utc;
use diesel::dsl::{any, sql};
use diesel::prelude::*;
use diesel::sql_types::BigInt;
use futures::future::Future;
use std::collections::HashMap;
use stratum_db::models::{Contest, ContestProblem, Problem, Team};
use stratum_db::schema::{contest_problems, judgements, problems, submissions, teams};
use stratum_db::Execute;
use tera::Context;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.route("", Method::GET, index)
}

pub fn index(
    req: HttpRequest<AppState>,
) -> Result<Box<(Future<Item = HttpResponse, Error = Error>)>, Error> {
    let (contest_id, contest_freeze) = req
        .extensions()
        .get::<Contest>()
        .map(|c| (c.id, c.freeze_at.unwrap_or_else(Utc::now)))
        .ok_or_else(|| error::ErrorInternalServerError("contest not bound"))?;
    Ok(req
        .state()
        .db
        .send(Execute::new(move |conn| -> Result<_, Error> {
            let cproblems = contest_problems::table
                .filter(contest_problems::contest_id.eq(contest_id))
                .inner_join(problems::table)
                .load::<(ContestProblem, Problem)>(&conn)
                .map_err(error::ErrorInternalServerError)?;
            let mut teams = teams::table
                .load::<Team>(&conn)
                .map_err(error::ErrorInternalServerError)?;
            let (pscores, tscores, bscores) = submissions::table
                .inner_join(
                    judgements::table.on(submissions::id
                        .eq(judgements::submission_id)
                        .and(submissions::location_id.eq(judgements::submission_location_id))),
                )
                .select((
                    submissions::team_id,
                    submissions::problem_id,
                    sql::<BigInt>("max(score)").nullable(),
                ))
                .filter(
                    submissions::problem_id
                        .eq(any(cproblems.iter().map(|i| i.1.id).collect::<Vec<_>>())),
                )
                .filter(
                    submissions::team_id.eq(any(teams.iter().map(|i| i.id).collect::<Vec<_>>())),
                )
                .filter(submissions::created_at.lt(contest_freeze))
                .filter(judgements::valid.eq(true))
                .group_by((submissions::problem_id, submissions::team_id))
                .load::<(i64, i64, Option<i64>)>(&conn)
                .map_err(error::ErrorInternalServerError)?
                .iter()
                // pscores: scores for each problem for each team
                // tscores: scores of each team
                // bscores: best scores of each problem
                .fold(
                    (HashMap::new(), HashMap::new(), HashMap::new()),
                    |(mut pscores, mut tscores, mut bscores), item| {
                        let score = item.2.unwrap_or(0);
                        if score > 0 {
                            let team = pscores.entry(item.0).or_insert_with(HashMap::new);
                            team.insert(item.1, score);
                            tscores
                                .entry(item.0)
                                .and_modify(|e| *e += score)
                                .or_insert(score);
                        }
                        bscores
                            .entry(item.1)
                            .and_modify(|e| {
                                if score > *e {
                                    *e = score
                                }
                            })
                            .or_insert(score);
                        (pscores, tscores, bscores)
                    },
                );
            teams.sort_by(|a, b| {
                tscores
                    .get(&b.id)
                    .unwrap_or(&0)
                    .cmp(&tscores.get(&a.id).unwrap_or(&0))
            });
            Ok((cproblems, teams, pscores, tscores, bscores))
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok((cproblems, teams, pscores, tscores, bscores)) => {
                let mut ctx = Context::new();
                ctx.insert("cproblems", &cproblems);
                ctx.insert("teams", &teams);
                ctx.insert("tscores", &tscores);
                ctx.insert("pscores", &pscores);
                ctx.insert("bscores", &bscores);
                render(&req, "contest/scoreboard/show.html", ctx)
            }
            Err(e) => Err(e),
        })
        .responder())
}
