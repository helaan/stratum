use crate::middleware::checkconteststarted::CheckContestStarted;
use crate::middleware::contestbinder::ContestBinder;
use crate::AppState;
use actix_web::Scope;

pub mod scoreboard;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.middleware(ContestBinder {})
        .middleware(CheckContestStarted {})
        .nested("", scoreboard::register)
}
