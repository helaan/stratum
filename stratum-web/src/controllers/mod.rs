use crate::AppState;
use actix_web::Scope;

pub mod admin;
pub mod contest;
pub mod legal;
pub mod overview;
pub mod session;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.nested("/admin", admin::register)
        .nested("/contest/{contest_name}", contest::register)
        .nested("/legal", legal::register)
        .nested("/session", session::register)
        .nested("", overview::register)
}
