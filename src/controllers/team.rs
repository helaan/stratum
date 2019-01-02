//! This controller handles team registration
//! - Create teams
//! - Show teams
//! - Edit teams
//! - Remove teams

use crate::database::{DbExecutor, Execute};
use crate::models::Team;
use crate::schema::teams;
use crate::{util::render, AppState};
use actix::prelude::*;
use actix_web::{error, AsyncResponder, Error, Form, HttpRequest, Path, Responder};
use diesel::prelude::*;
use futures::future::Future;
use std::ops::Deref;
use tera::Context;

#[derive(Deserialize, Insertable)]
#[table_name = "teams"]
pub struct CreateTeam {
    name: String,
}

impl Message for CreateTeam {
    type Result = Result<(), actix_web::Error>;
}

impl Handler<CreateTeam> for DbExecutor {
    type Result = Result<(), actix_web::Error>;
    fn handle(&mut self, team: CreateTeam, _: &mut Self::Context) -> Self::Result {
        diesel::insert_into(teams::table)
            .values(&team)
            .execute(self.get_conn()?.deref())
            .map(|_| ())
            .map_err(|_| error::ErrorInternalServerError("Error inserting team"))
    }
}

pub fn create_form(req: HttpRequest<AppState>) -> impl Responder {
    let ctx = Context::new();
    render(req.state(), "team/create.html", &ctx)
}

pub fn create(req: HttpRequest<AppState>, form: Form<CreateTeam>) -> impl Responder {
    //req.state().db.send(form.into_inner()).from_err().and_then(move |res| match res {
    req.state()
        .db
        .send(Execute::new(|s| -> Result<usize, Error> {
            let conn = s.get_conn()?;
            diesel::insert_into(teams::table)
                .values(&form.into_inner())
                .execute(&conn)
                .map_err(|_| error::ErrorInternalServerError("Error inserting team"))
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok(_) => Ok("succesfully added team"),
            Err(e) => Err(e),
        })
        .responder()
}

pub fn index(req: HttpRequest<AppState>) -> impl Responder {
    req.state()
        .db
        .send(Execute::new(|s| -> Result<Vec<Team>, Error> {
            let conn = s.get_conn()?;
            teams::dsl::teams
                .order(teams::id.asc())
                .load(&conn)
                .map_err(|e| error::ErrorInternalServerError(e))
        }))
        .from_err()
        .and_then(move |res: Result<Vec<Team>, Error>| match res {
            Ok(teams) => {
                let mut ctx = Context::new();
                ctx.insert("teams", &teams);
                render(req.state(), "team/index.html", &ctx)
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
    req.state()
        .db
        .send(Execute::new(move |s| -> Result<Team, Error> {
            let conn = s.get_conn()?;
            teams::dsl::teams
                .find(params.id)
                .get_result::<Team>(&conn)
                .map_err(|e| error::ErrorInternalServerError(e))
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok(team) => {
                let mut ctx = Context::new();
                ctx.insert("team", &team);
                render(req.state(), "team/show.html", &ctx)
            }
            Err(e) => Err(e),
        })
        .responder()
}

pub fn edit(
    req: HttpRequest<AppState>,
    params: Path<IdParams>,
    form: Form<Team>,
) -> impl Responder {
    let team = form.into_inner();
    req.state()
        .db
        .send(Execute::new(move |s| -> Result<usize, Error> {
            let conn = s.get_conn()?;
            if team.id != params.id {
                return Err(error::ErrorBadRequest(format!(
                    "Attempted to update different team, expected {}, was given {}",
                    team.id, params.id
                )));
            }
            diesel::update(&team)
                .set(&team)
                .execute(&conn)
                .map_err(|e| error::ErrorInternalServerError(e))
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok(rows) => Ok(format!("Updated {} team", rows)),
            Err(e) => Err(e),
        })
        .responder()
}

pub fn destroy() {}
