use crate::AppState;
use actix_web::middleware::{Middleware, Started};
use actix_web::{error, Error, HttpRequest};
use diesel::prelude::*;
use diesel::sql_types::Text;
use futures::future::{err, ok, Future};
use stratum_db::models::Contest;
use stratum_db::schema::contests;
use stratum_db::Execute;

sql_function!(fn lower(x: Text) -> Text);

pub struct ContestBinder;

impl Middleware<AppState> for ContestBinder {
    fn start(&self, req: &HttpRequest<AppState>) -> Result<Started, Error> {
        let contest_name = req.match_info().query::<String>("contest_name")?;
        let req = req.clone();
        Ok(Started::Future(Box::new(
            req.state()
                .db
                .send(Execute::new(move |conn| {
                    contests::table
                        .filter(lower(contests::short_name).eq(contest_name.to_lowercase()))
                        .first::<Contest>(&conn)
                        .optional()
                        .map_err(error::ErrorInternalServerError)
                }))
                .from_err()
                .and_then(move |res| match res {
                    Ok(opt) => {
                        if let Some(contest) = opt {
                            req.extensions_mut().insert(contest);
                            ok(None)
                        } else {
                            err(error::ErrorNotFound(
                                "Could not find contest with this name",
                            ))
                        }
                    }
                    Err(e) => err(e),
                }),
        )))
    }
}
