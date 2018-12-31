//! This controller handles team registration
//! - Create teams
//! - Show teams
//! - Edit teams
//! - Remove teams

use actix_web::{error, AsyncResponder, Form, HttpRequest, Responder};
use actix::prelude::*;
use crate::{AppState, util::render};
use crate::database::DbExecutor;
use tera::{Context};
use crate::schema::teams;
use crate::models::Team;
use std::ops::Deref;
use diesel::prelude::*;
use futures::future::Future;

#[derive(Deserialize, Insertable)]
#[table_name = "teams"]
pub struct CreateTeam {
    name: String
}

impl Message for CreateTeam {
    type Result = Result<(), actix_web::Error>;
}

impl Handler<CreateTeam> for DbExecutor {
    type Result = Result<(), actix_web::Error>;
    fn handle(&mut self, team: CreateTeam, _: &mut Self::Context) -> Self::Result {
        diesel::insert_into(teams::table).values(&team).execute(self.get_conn()?.deref())
            .map(|_| ())
            .map_err(|_| error::ErrorInternalServerError("Error inserting team"))
    }
}



pub fn create_form(req: HttpRequest<AppState>) -> impl Responder {
    let ctx = Context::new();
    render(req.state(), "team/create.html", &ctx)
}

pub fn create(req: HttpRequest<AppState>, form: Form<CreateTeam> )  -> impl Responder {
    req.state().db.send(form.into_inner()).from_err().and_then(move |res| match res {
        Ok(_) => {
            Ok("succesfully added team")
        }
        Err(e) => Err(e)
    }).responder()
}

pub struct AllTeams;

impl Message for AllTeams {
    type Result = Result<Vec<Team>, actix_web::Error>;
}

impl Handler<AllTeams> for DbExecutor {
    type Result = Result<Vec<Team>,actix_web::Error>;
    fn handle(&mut self, _: AllTeams, _: &mut Self::Context) -> Self::Result {
        let conn = self.get_conn()?;
        teams::dsl::teams.order(teams::id.asc()).load::<Team>(&conn)
            .map_err(|e| error::ErrorInternalServerError(e))
    }
}

pub fn index(req: HttpRequest<AppState>) -> impl Responder {
    req.state().db.send(AllTeams).from_err().and_then(move |res| match res {
        Ok(teams) => {
            let mut ctx = Context::new();
            ctx.insert("teams", &teams);
            render(req.state(), "team/index.html", &ctx)
        },
        Err(e) => Err(e)
    }).responder()
}

//pub fn show(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
//   
//}

pub fn edit() {

}

pub fn destroy() {

}
