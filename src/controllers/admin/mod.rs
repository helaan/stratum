use crate::middleware::checkrights::CheckRights;
use crate::AppState;
use actix_web::Scope;

pub mod overview;
pub mod problem_statement;
pub mod team;
pub mod user;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.middleware(CheckRights { min_rights: 1000 })
        .nested("/problem_statement", problem_statement::register)
        .nested("/team", team::register)
        .nested("/user", user::register)
        .nested("", overview::register)
}
