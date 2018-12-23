// Due to Diesel issues, disable this warning.
// See https://github.com/diesel-rs/diesel/issues/1785
#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use] extern crate diesel;

extern crate actix;
extern crate actix_web;
extern crate dotenv;

use actix_web::{App,HttpRequest,server};
use dotenv::dotenv;

mod models;
mod schema;
//mod config;
//mod database;

fn index(_req: &HttpRequest) -> &'static str {
    "Hello world!"
}

fn main() {
    dotenv().ok();
   // let cfg = config::get();

    server::new( || App::new().resource("/", |r| r.f(index)))
        .bind("127.0.0.1:8008")
        .unwrap()
        .run();
}

