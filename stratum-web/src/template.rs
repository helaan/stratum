use crate::AppState;
use actix_web::HttpRequest;
use chrono::{DateTime, Local};
use stratum_db::models::{Contest, Team, User};

pub struct TemplateContext {
    pub user: Option<User>,
    pub team: Option<Team>,
    pub render_time: DateTime<Local>,
}

impl TemplateContext {
    pub fn new(req: &HttpRequest<AppState>) -> Self {
        let tmp_req = req.clone();
        let mut extensions = tmp_req.extensions_mut();
        TemplateContext {
            user: extensions.remove::<User>(),
            team: extensions.remove::<Team>(),
            render_time: Local::now(),
        }
    }
}

pub fn extract_contest(req: &HttpRequest<AppState>) -> Option<Contest> {
    let tmp_req = req.clone();
    let mut extensions = tmp_req.extensions_mut();
    extensions.remove::<Contest>()
}
