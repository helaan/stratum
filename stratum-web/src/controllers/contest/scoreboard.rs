use crate::template::{extract_contest, TemplateContext};
use crate::AppState;
use actix_web::{error, http::Method, AsyncResponder, Error, HttpRequest, Responder, Scope};
use askama::Template;
use chrono::Utc;
use diesel::dsl::{any, sql};
use diesel::prelude::*;
use diesel::sql_types::BigInt;
use futures::future::Future;
use std::borrow::Borrow;
use std::collections::HashMap;
use stratum_db::models::{Contest, ContestProblem, Problem, Team, User};
use stratum_db::schema::{contest_problems, judgements, problems, submissions, teams};
use stratum_db::Execute;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.route("", Method::GET, index)
}

#[derive(Template)]
#[template(path = "contest/scoreboard/index.html")]
struct IndexTemplate {
    ctx: TemplateContext,
    contest: Contest,
    cproblems: Vec<(ContestProblem, Problem)>,
    teams: Vec<(Team, i64, HashMap<i64, i64>)>,
    bscores: HashMap<i64, i64>,
}

pub fn index(
    req: HttpRequest<AppState>,
) -> Result<Box<(Future<Item = impl Responder, Error = Error>)>, Error> {
    let (contest_id, mut contest_freeze, contest_end) = req
        .extensions()
        .get::<Contest>()
        .map(|c| (c.id, c.freeze_at.unwrap_or_else(Utc::now), c.end_at))
        .ok_or_else(|| error::ErrorInternalServerError("contest not bound"))?;
    if let Some(user) = req.extensions().get::<User>() {
        if user.rights >= 1000 {
            contest_freeze = contest_end.unwrap_or_else(Utc::now);
        }
    }
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
                .map_err(error::ErrorInternalServerError)?
                .into_iter()
                .map(|t| (t.id, (t, 0, HashMap::new())))
                .collect::<HashMap<_, _>>();
            let bscores = submissions::table
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
                .filter(submissions::team_id.eq(any(teams.keys().collect::<Vec<_>>())))
                .filter(submissions::created_at.lt(contest_freeze))
                .filter(judgements::valid.eq(true))
                .group_by((submissions::problem_id, submissions::team_id))
                .load::<(i64, i64, Option<i64>)>(&conn)
                .map_err(error::ErrorInternalServerError)?
                .into_iter()
                // pscores: scores for each problem for each team (teamid -> problem_id -> score)
                // tscores: scores of each team (team_id -> score)
                // bscores: best scores of each problem (problem_id -> score)
                .fold(
                    HashMap::new(),
                    |mut bscores, (team_id, problem_id, max_score)| {
                        let score = max_score.unwrap_or(0);
                        if score > 0 {
                            let team = teams.get_mut(&team_id).unwrap(); // TODO fix
                            team.2.insert(problem_id, score);
                            team.1 += score;
                        }
                        bscores
                            .entry(problem_id)
                            .and_modify(|e| {
                                if score > *e {
                                    *e = score
                                }
                            })
                            .or_insert(score);
                        bscores
                    },
                );
            let mut teams_sorted = teams.into_iter().map(|t| t.1).collect::<Vec<_>>();
            teams_sorted.sort_by(|a, b| b.1.cmp(&a.1));
            Ok((cproblems, teams_sorted, bscores))
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok((cproblems, teams, bscores)) => Ok(IndexTemplate {
                ctx: TemplateContext::new(&req),
                contest: extract_contest(&req)
                    .ok_or_else(|| error::ErrorInternalServerError("contest not bound"))?,

                cproblems,
                teams,
                bscores,
            }),
            Err(e) => Err(e),
        })
        .responder())
}
