// Due to Diesel issues, disable this warning.
// See https://github.com/diesel-rs/diesel/issues/1785
#![allow(proc_macro_derive_resolution_fallback)]

// Diesel macros are needed in autogenerated schema
#[macro_use]
extern crate diesel;

use actix::prelude::*;
use actix_web::middleware::{session::*, Logger};
use actix_web::{fs, server, App};
use dotenv::dotenv;
use std::env;
use tera::{compile_templates, Tera};

mod controllers;
mod database;
mod middleware;
mod models;
mod pass;
mod schema;
mod util;

pub struct AppState {
    pub template: Tera,
    pub db: Addr<database::DbExecutor>,
}

fn main() {
    dotenv().ok();
    env_logger::init();
    log::debug!("Stratum starting...");
    let system = actix::System::new("stratum");

    // Database initialization
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let db_pool = database::create_pool(&db_url).expect("Could not create pool");
    let db_addr = SyncArbiter::start(3, move || database::DbExecutor(db_pool.clone()));

    let cookie_key = env::var("COOKIE_KEY").expect("COOKIE_KEY not set");
    let cookie_secure = env::var("COOKIE_SECURE").expect("COOKIE_SECURE not set") == "TRUE";

    let app = move || {
        let templates = compile_templates!("templates/**/*.html");

        let state = AppState {
            template: templates,
            db: db_addr.clone(),
        };

        App::with_state(state)
            .middleware(Logger::default())
            .middleware(SessionStorage::new(
                CookieSessionBackend::private(&cookie_key.as_bytes())
                    .http_only(true)
                    .secure(cookie_secure),
            ))
            .middleware(middleware::DataBinder {})
            .handler("/static", fs::StaticFiles::new("./static").unwrap())
            .scope("/", controllers::register)
    };

    server::new(app).bind("127.0.0.1:8008").unwrap().start();

    let _ = system.run();
}
