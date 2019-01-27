// Diesel macros are needed in some struct generated throughout the app
// TODO switch to 2018-style imports
#[macro_use]
extern crate diesel;

use actix::prelude::*;
use actix_web::middleware::{session::*, Logger};
use actix_web::{fs, server, App};
use dotenv::dotenv;
use std::env;
use stratum_db::{create_pool, DbExecutor};
use tera::{compile_templates, Tera};

mod controllers;
mod error_pages;
mod middleware;
mod multipart;
mod pass;
mod util;

pub struct AppState {
    pub template: Tera,
    pub db: Addr<DbExecutor>,
}

fn main() {
    dotenv().ok();
    env_logger::init();
    log::debug!("Stratum starting...");
    let system = actix::System::new("stratum");

    // Database initialization
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let db_pool = create_pool(&db_url).expect("Could not create pool");
    let db_addr = SyncArbiter::start(3, move || DbExecutor(db_pool.clone()));

    let cookie_key = env::var("COOKIE_KEY").expect("COOKIE_KEY not set");
    let cookie_secure = env::var("COOKIE_SECURE").expect("COOKIE_SECURE not set") == "TRUE";

    let app = move || {
        let templates = compile_templates!("./stratum-web/templates/**/*.html");

        let state = AppState {
            template: templates,
            db: db_addr.clone(),
        };

        App::with_state(state)
            .middleware(Logger::default())
            .middleware(error_pages::register())
            .middleware(SessionStorage::new(
                CookieSessionBackend::private(&cookie_key.as_bytes())
                    .http_only(true)
                    .secure(cookie_secure),
            ))
            .middleware(middleware::databinder::DataBinder {})
            .handler(
                "/static",
                fs::StaticFiles::new("./stratum-web/static").unwrap(),
            )
            .scope("/", controllers::register)
    };

    server::new(app).bind("127.0.0.1:8008").unwrap().start();

    let _ = system.run();
}
