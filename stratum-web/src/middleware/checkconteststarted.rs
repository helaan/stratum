use crate::AppState;
use actix_web::middleware::{Middleware, Started};
use actix_web::{error, Error, HttpRequest};
use chrono::Utc;
use stratum_db::models::Contest;

pub struct CheckContestStarted;

impl Middleware<AppState> for CheckContestStarted {
    fn start(&self, req: &HttpRequest<AppState>) -> Result<Started, Error> {
        if let Some(contest) = req.extensions().get::<Contest>() {
            if let Some(start_at) = contest.start_at {
                if Utc::now() >= start_at {
                    Ok(Started::Done)
                } else {
                    Err(error::ErrorForbidden(format!(
                        "This contest will start at {}",
                        start_at
                    )))
                }
            } else {
                // No start time -> contest not published
                Err(error::ErrorNotFound(""))
            }
        } else {
            Err(error::ErrorNotFound(""))
        }
    }
}
