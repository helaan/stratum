//! This controller handles team registration
//! - Create teams
//! - Show teams
//! - Edit teams
//! - Remove teams (TODO)

use crate::template::TemplateContext;
use crate::AppState;
use actix_web::{
    error, http::Method, AsyncResponder, Error, Form, HttpRequest, Path, Responder, Scope,
};
use askama::Template;
use diesel::prelude::*;
use diesel::Insertable;
use futures::future::Future;
use serde::Deserialize;
use stratum_db::models::Team;
use stratum_db::schema::teams;
use stratum_db::Execute;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.route("", Method::GET, index)
        .route("/new", Method::GET, create_form)
        .route("/new", Method::POST, create)
        .route("/{id:\\d+}", Method::GET, show)
        .route("/{id:\\d+}", Method::POST, edit)
}

#[derive(Deserialize, Insertable)]
#[table_name = "teams"]
pub struct CreateTeam {
    name: String,
}

#[derive(Template)]
#[template(path = "admin/team/create.html")]
struct CreateFormTemplate {
    ctx: TemplateContext,
}

pub fn create_form(req: HttpRequest<AppState>) -> impl Responder {
    CreateFormTemplate {
        ctx: TemplateContext::new(&req),
    }
}

pub fn create(req: HttpRequest<AppState>, form: Form<CreateTeam>) -> impl Responder {
    req.state()
        .db
        .send(Execute::new(|conn| -> Result<usize, Error> {
            diesel::insert_into(teams::table)
                .values(&form.into_inner())
                .execute(&conn)
                .map_err(error::ErrorInternalServerError)
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok(_) => Ok("succesfully added team"),
            Err(e) => Err(e),
        })
        .responder()
}

#[derive(Template)]
#[template(path = "admin/team/index.html")]
struct IndexTemplate {
    ctx: TemplateContext,
    teams: Vec<Team>,
}

pub fn index(req: HttpRequest<AppState>) -> impl Responder {
    req.state()
        .db
        .send(Execute::new(|conn| -> Result<Vec<Team>, Error> {
            teams::dsl::teams
                .order(teams::id.asc())
                .load(&conn)
                .map_err(error::ErrorInternalServerError)
        }))
        .from_err()
        .and_then(move |res: Result<Vec<Team>, Error>| match res {
            Ok(teams) => Ok(IndexTemplate {
                ctx: TemplateContext::new(&req),
                teams,
            }),
            Err(e) => Err(e),
        })
        .responder()
}

#[derive(Deserialize)]
pub struct IdParams {
    id: i64,
}

#[derive(Template)]
#[template(path = "admin/team/show.html")]
struct ShowTemplate {
    ctx: TemplateContext,
    team: Team,
}

pub fn show(req: HttpRequest<AppState>, params: Path<IdParams>) -> impl Responder {
    req.state()
        .db
        .send(Execute::new(move |conn| -> Result<Team, Error> {
            teams::dsl::teams
                .find(params.id)
                .get_result::<Team>(&conn)
                .map_err(error::ErrorInternalServerError)
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok(team) => Ok(ShowTemplate {
                ctx: TemplateContext::new(&req),
                team,
            }),
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
        .send(Execute::new(move |conn| -> Result<usize, Error> {
            if team.id != params.id {
                return Err(error::ErrorBadRequest(format!(
                    "Attempted to update different team, expected {}, was given {}",
                    team.id, params.id
                )));
            }
            diesel::update(&team)
                .set(&team)
                .execute(&conn)
                .map_err(error::ErrorInternalServerError)
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok(rows) => Ok(format!("Updated {} team", rows)),
            Err(e) => Err(e),
        })
        .responder()
}

//pub fn destroy() {}
