use crate::database::Execute;
use crate::models::{Session, Team, User};
use crate::schema::{sessions, teams, users};
use crate::AppState;
use actix_web::middleware::session::RequestSession;
use actix_web::middleware::{Middleware, Started};
use actix_web::{error, Error, HttpRequest};
use diesel::prelude::*;
use futures::future::{err, ok, Future};
use uuid::Uuid;

/** DataBinder looks at the session cookie and preloads the associated user
 * and team.
 *
 * They are stored in the request extensions.
 *
 * This middleware must be called after the CookieSessionBackend, as it uses
 * cookies, but before any middleware that uses the results.
 */
pub struct DataBinder;

impl Middleware<AppState> for DataBinder {
    fn start(&self, req: &HttpRequest<AppState>) -> Result<Started, Error> {
        let cookie_key = req.session().get::<Uuid>("key");
        if let Some(key) = cookie_key.unwrap() {
            let req = req.clone();
            let fut = req
                .state()
                .db
                .send(Execute::new(
                    move |s| {
                        let conn = s.get_conn()?;
                        sessions::dsl::sessions
                            .find(key)
                            .left_join(users::dsl::users.left_join(teams::dsl::teams))
                            .first::<(Session, Option<(User, Option<Team>)>)>(&conn)
                            .map_err(error::ErrorInternalServerError)
                    },
                ))
                .from_err()
                .and_then(
                    move |res| match res {
                        Ok(tup) => {
                            if let Some(utup) = tup.1 {
                                req.extensions_mut().insert(utup.0);
                                if let Some(team) = utup.1 {
                                    req.extensions_mut().insert(team);
                                }
                            }
                            ok(None)
                        }
                        Err(e) => err(e),
                    },
                );
            Ok(Started::Future(Box::new(fut)))
        } else {
            Ok(Started::Done)
        }
    }
}
