#[macro_use] extern crate serde_derive;

extern crate actix_web;
use actix_web::{App,HttpRequest,server};

mod config;

fn index(_req: &HttpRequest) -> &'static str {
    "Hello world!"
}

fn main() {
    config::get();
    server::new( || App::new().resource("/", |r| r.f(index)))
        .bind("127.0.0.1:8008")
        .unwrap()
        .run();
}

