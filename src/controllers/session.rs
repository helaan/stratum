use crate::database::Execute;
use crate::models::{Session, User};
use crate::schema::{sessions, users};
use crate::util::render;
use crate::{pass, AppState};
use actix_web::middleware::session::RequestSession;
use actix_web::{error, http::Method, AsyncResponder, Error, Form, HttpRequest, Responder, Scope};
use diesel::prelude::*;
use futures::future::Future;
use serde::Deserialize;
use tera::Context;
use uuid::Uuid;

/** Responsible for session management
 * - Login
 * - Logout
 */

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.route("/login", Method::GET, login_form)
        .route("/login", Method::POST, login)
        .route("/logout", Method::POST, logout)
}

pub fn login_form(req: HttpRequest<AppState>) -> impl Responder {
    let ctx = Context::new();
    render(&req, "session/login.html", ctx)
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

pub fn login(req: HttpRequest<AppState>, form: Form<LoginForm>) -> impl Responder {
    req.state().db.send(Execute::new(move |conn| -> Result<Vec<Session>, Error> {
        let query : Result<Option<User>, diesel::result::Error> = users::dsl::users.filter(users::columns::username.eq(&form.username)).first(&conn).optional();
        match query {
            Ok(opt_user) => {
                // FUTURE: when let_chains stabilize, clean this up
                if let Some(user) = opt_user {
                    return match pass::check(&user.password_hash, &form.password) {
                        Ok(correct) => if correct {
                            diesel::insert_into(sessions::table).values((sessions::columns::user_id.eq(user.id), sessions::columns::key.eq(Uuid::new_v4()))).get_results(&conn)
                                .map_err(|e| {error::ErrorInternalServerError(e)})
                        } else {
                            Err(error::ErrorUnauthorized("Could not log in: please check your username/password combination"))
                        }
                        Err(e) => Err(error::ErrorInternalServerError(e))
                    }
                } else {
                    Err(error::ErrorUnauthorized("Could not log in: please check your username/password combination"))
                }
            },
            Err(e) => Err(error::ErrorInternalServerError(e))
        }

    })).from_err()
    .and_then(move |res| match res {
        Ok(sessions) => {
            if sessions.len() != 1 {
                return Err(error::ErrorInternalServerError(format!("Received {} sessions from database, expected 1", sessions.len())));
            }
            req.session().set("key", sessions[0].key)?;
            Ok("logged in successfully")
        }
        Err(e) => Err(e)
    }).responder()
}

pub fn logout(req: HttpRequest<AppState>) -> impl Responder {
    req.session().remove("key");
    "logged out successfully"
}
