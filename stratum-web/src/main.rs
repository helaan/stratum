// Diesel macros are needed in some struct generated throughout the app
// TODO switch to 2018-style imports
#[macro_use]
extern crate diesel;

use actix::prelude::*;
use actix_web::middleware::{session::*, Logger};
use actix_web::{fs, server, App};
use dotenv::dotenv;
use sentry_actix::SentryMiddleware;
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
    pub location_id: i32,
}

fn main() {
    dotenv().ok();

    let sentry = env::var("SENTRY_DSN").map(sentry::init);
    let sentry_enabled = sentry.is_ok();

    if sentry_enabled {
        sentry::integrations::panic::register_panic_handler();
        sentry::integrations::env_logger::init(None, Default::default());
        log::info!("Sentry initialized!");
    } else {
        env_logger::init();
        log::info!("No SENTRY_DSN found, not registering with Sentry");
    }
    log::debug!("Stratum starting...");
    let system = actix::System::new("stratum");

    // Database initialization
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let db_pool = create_pool(&db_url).expect("Could not create pool");
    let db_addr = SyncArbiter::start(3, move || DbExecutor(db_pool.clone()));

    let cookie_key = env::var("COOKIE_KEY").expect("COOKIE_KEY not set");
    let cookie_secure = env::var("COOKIE_SECURE").expect("COOKIE_SECURE not set") == "TRUE";

    let location_id = env::var("LOCATION_ID")
        .expect("LOCATION_ID not set")
        .parse()
        .unwrap();

    let app = move || {
        let templates = compile_templates!("./stratum-web/templates/**/*.html");

        let state = AppState {
            template: templates,
            db: db_addr.clone(),
            location_id,
        };

        let mut app = App::with_state(state);
        if sentry_enabled {
            app = app.middleware(SentryMiddleware::new());
        }
        app.middleware(Logger::default())
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
