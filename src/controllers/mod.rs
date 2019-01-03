use crate::AppState;
use actix_web::Scope;

pub mod admin;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.nested("/admin", admin::register)
}
