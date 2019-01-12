use crate::AppState;
use actix_web::Scope;
use crate::middleware::checkrights::CheckRights;

pub mod team;
pub mod user;
pub mod overview;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.middleware(CheckRights { min_rights: 1000 })
        .nested("/team", team::register)
        .nested("/user", user::register)
        .nested("", overview::register)
}
