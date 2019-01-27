use crate::multipart::parse_multipart;
use crate::util::render;
use crate::AppState;
use actix_web::{error, http::Method, AsyncResponder, HttpMessage, HttpRequest, Responder, Scope};
use diesel::prelude::*;
use futures::future::Future;
use stratum_db::models::{Problem, TestCase};
use stratum_db::schema::{problems, test_cases};
use stratum_db::Execute;
use tera::Context;

pub fn register(scop: Scope<AppState>) -> Scope<AppState> {
    scop.route("", Method::GET, index)
        .route("/new", Method::GET, create_form)
        .route("/new", Method::POST, create)
}

pub fn create(req: HttpRequest<AppState>) -> impl Responder {
    let mp = parse_multipart(req.multipart());
    mp.then(move |x| {
        let v = x.unwrap();
        req.state().db.send(Execute::new(move |conn| {
            let problem_id: i64 = v.get_parsed_content("problem_id")?;
            let position: i32 = v.get_parsed_content("position")?;
            let description = v.get_content_str("description")?;
            let input = v.get("input")?;
            let output = v.get("output")?;
            let visible_rights: i16 = v.get_parsed_content("visible_rights")?;

            diesel::insert_into(test_cases::table)
                .values((
                    test_cases::problem_id.eq(problem_id),
                    test_cases::position.eq(position),
                    test_cases::description.eq(description),
                    test_cases::input.eq(input.2.as_ref()),
                    test_cases::input_mimetype.eq(&input.0),
                    test_cases::output.eq(output.2.as_ref()),
                    test_cases::output_mimetype.eq(&output.0),
                    test_cases::visible_rights.eq(visible_rights),
                ))
                .execute(&conn)
                .map_err(error::ErrorInternalServerError)
        }))
    })
    .from_err()
    .and_then(|res| match res {
        Ok(_) => Ok("uploaded test case"),
        Err(e) => Err(e),
    })
    .responder()
}

pub fn create_form(req: HttpRequest<AppState>) -> impl Responder {
    render(&req, "admin/test_case/create.html", Context::new())
}

pub fn index(req: HttpRequest<AppState>) -> impl Responder {
    req.state()
        .db
        .send(Execute::new(|conn| {
            test_cases::table
                .inner_join(problems::table)
                .order((problems::id.asc(), test_cases::position.asc()))
                .load::<(TestCase, Problem)>(&conn)
                .map_err(error::ErrorInternalServerError)
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok(problem_statements) => {
                let mut ctx = Context::new();
                ctx.insert("test_cases", &problem_statements);
                render(&req, "admin/test_case/index.html", ctx)
            }
            Err(e) => Err(e),
        })
        .responder()
}
