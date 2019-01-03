use crate::AppState;
use actix_web::Scope;

pub mod team;
pub mod user;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.nested("/team", team::register)
        .nested("/user", user::register)
}
