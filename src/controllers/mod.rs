use crate::AppState;
use actix_web::Scope;

pub mod admin;
pub mod scoreboard;
pub mod session;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.nested("/admin", admin::register)
        .nested("/session", session::register)
        .nested("", scoreboard::register)
}
