// Due to Diesel issues, disable this warning.
// See https://github.com/diesel-rs/diesel/issues/1785
#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

use actix::prelude::*;
use actix_web::middleware::Logger;
use actix_web::{http, server, App};
use dotenv::dotenv;
use std::env;
use tera::{compile_templates, Tera};

mod controllers;
mod database;
mod models;
mod schema;
mod util;

pub struct AppState {
    pub template: Tera,
    pub db: Addr<database::DbExecutor>,
}

fn main() {
    dotenv().ok();
    env_logger::init();
    debug!("Calamaris starting...");
    let system = actix::System::new("calamaris");

    // Database initialization
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let db_pool = database::create_pool(&db_url).expect("Could not create pool");
    let db_addr = SyncArbiter::start(3, move || database::DbExecutor(db_pool.clone()));

    let app = move || {
        let templates = compile_templates!("templates/**/*.html");

        let state = AppState {
            template: templates,
            db: db_addr.clone(),
        };

        App::with_state(state)
            .middleware(Logger::default())
            .route("/admin/team", http::Method::GET, controllers::team::index)
            .route(
                "/admin/team/new",
                http::Method::GET,
                controllers::team::create_form,
            )
            .route(
                "/admin/team/new",
                http::Method::POST,
                controllers::team::create,
            )
            .route(
                "/admin/team/{id:\\d+}",
                http::Method::GET,
                controllers::team::show,
            )
            .route(
                "/admin/team/{id:\\d+}",
                http::Method::POST,
                controllers::team::edit,
            )
    };

    server::new(app).bind("127.0.0.1:8008").unwrap().start();

    let _ = system.run();
}
