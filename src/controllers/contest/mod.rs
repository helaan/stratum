use crate::middleware::checkconteststarted::CheckContestStarted;
use crate::middleware::contestbinder::ContestBinder;
use crate::AppState;
use actix_web::Scope;

pub mod problem;
pub mod scoreboard;
pub mod submission;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.middleware(ContestBinder {})
        .middleware(CheckContestStarted {})
        .nested("/problem", problem::register)
        .nested("/submission", submission::register)
        .nested("", scoreboard::register)
}
