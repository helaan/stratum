//! This controller handles user registration
//! - Create users
//! - Show users
//! - Edit users
//! - Remove users (TODO)

use crate::database::Execute;
use crate::models::{Team, User};
use crate::pass::hash;
use crate::schema::{sessions, teams, users};
use crate::{util::render, AppState};
use actix_web::{
    error, http::Method, AsyncResponder, Error, Form, HttpRequest, Path, Responder, Scope,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::{dsl::sql, sql_types::Timestamptz, AsChangeset, Insertable};
use futures::future::Future;
use serde::Deserialize;
use std::collections::HashMap;
use tera::Context;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.route("", Method::GET, index)
        .route("/new", Method::GET, create_form)
        .route("/new", Method::POST, create)
        .route("/{id:\\d+}", Method::GET, show)
        .route("/{id:\\d+}", Method::POST, edit)
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct CreateUser {
    team_id: Option<i64>,
    username: String,
    password_hash: String,
    rights: i16,
}

#[derive(Deserialize)]
pub struct CreateUserForm {
    team_id: String, // Manually parse :(
    username: String,
    password: String,
    rights: i16,
}

pub fn create_form(req: HttpRequest<AppState>) -> impl Responder {
    let ctx = Context::new();
    render(&req, "admin/user/create.html", ctx)
}

pub fn create(req: HttpRequest<AppState>, form: Form<CreateUserForm>) -> impl Responder {
    let f = form.into_inner();
    let user = CreateUser {
        team_id: f.team_id.parse::<i64>().ok(),
        username: f.username,
        password_hash: hash(f.password),
        rights: f.rights,
    };
    req.state()
        .db
        .send(Execute::new(move |s| -> Result<usize, Error> {
            let conn = s.get_conn()?;
            diesel::insert_into(users::table)
                .values(&user)
                .execute(&conn)
                .map_err(|_| error::ErrorInternalServerError("Error inserting user"))
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok(_) => Ok("succesfully added user"),
            Err(e) => Err(e),
        })
        .responder()
}

pub fn index(req: HttpRequest<AppState>) -> impl Responder {
    req.state()
        .db
        .send(Execute::new(
            |s| -> Result<(Vec<(User, Option<Team>)>, HashMap<i64, DateTime<Utc>>), Error> {
                let conn = s.get_conn()?;
                let users = users::dsl::users
                    .left_join(teams::dsl::teams)
                    .order(users::id.asc())
                    .load(&conn)
                    .map_err(|e| error::ErrorInternalServerError(e))?;
                let login_times = sessions::table
                    .select((sessions::user_id, sql::<Timestamptz>("max(created_at)")))
                    .group_by(sessions::user_id)
                    .load(&conn)
                    .map(|d| d.iter().cloned().collect())
                    .map_err(|e| error::ErrorInternalServerError(e))?;
                Ok((users, login_times))
            },
        ))
        .from_err()
        .and_then(move |res| match res {
            Ok((users, last_login_times)) => {
                let mut ctx = Context::new();
                ctx.insert("users", &users);
                ctx.insert("last_login_times", &last_login_times);
                render(&req, "admin/user/index.html", ctx)
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
        .send(Execute::new(move |s| -> Result<User, Error> {
            let conn = s.get_conn()?;
            users::dsl::users
                .find(params.id)
                .get_result::<User>(&conn)
                .map_err(|e| error::ErrorInternalServerError(e))
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok(user) => {
                let mut ctx = Context::new();
                ctx.insert("user", &user);
                render(&req, "admin/user/show.html", ctx)
            }
            Err(e) => Err(e),
        })
        .responder()
}

#[derive(AsChangeset, Identifiable)]
#[table_name = "users"]
pub struct EditUser {
    id: i64,
    team_id: Option<i64>,
    username: String,
    password_hash: Option<String>,
    rights: i16,
}
#[derive(Deserialize)]
pub struct EditUserForm {
    team_id: String, // Manually parse :(
    username: String,
    password: String,
    rights: i16,
}

pub fn edit(
    req: HttpRequest<AppState>,
    params: Path<IdParams>,
    form: Form<EditUserForm>,
) -> impl Responder {
    let f = form.into_inner();
    let new_pass = if f.password.is_empty() {
        None
    } else {
        Some(hash(f.password))
    };
    let user = EditUser {
        id: params.id,
        team_id: f.team_id.parse::<i64>().ok(),
        username: f.username,
        password_hash: new_pass,
        rights: f.rights,
    };
    req.state()
        .db
        .send(Execute::new(move |s| -> Result<usize, Error> {
            let conn = s.get_conn()?;
            if user.id != params.id {
                return Err(error::ErrorBadRequest(format!(
                    "Attempted to update different user, expected {}, was given {}",
                    user.id, params.id
                )));
            }
            diesel::update(&user)
                .set(&user)
                .execute(&conn)
                .map_err(|e| error::ErrorInternalServerError(e))
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok(rows) => Ok(format!("Updated {} user", rows)),
            Err(e) => Err(e),
        })
        .responder()
}
