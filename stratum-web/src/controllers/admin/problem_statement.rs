use crate::database::Execute;
use crate::models::{Problem, ProblemStatement};
use crate::multipart::parse_multipart;
use crate::schema::{problem_statements, problems};
use crate::util::render;
use crate::AppState;
use actix_web::{error, http::Method, AsyncResponder, HttpMessage, HttpRequest, Responder, Scope};
use diesel::prelude::*;
use futures::future::Future;
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
            let statement = v.get("statement")?;
            let problem_id: i64 = v.get_parsed_content("problem_id")?;
            diesel::insert_into(problem_statements::table)
                .values((
                    problem_statements::problem_id.eq(problem_id),
                    problem_statements::filename
                        .eq(statement.1.get_filename().unwrap_or("unnamed file")),
                    problem_statements::mimetype.eq(&statement.0),
                    problem_statements::statement.eq(statement.2.as_ref()),
                ))
                .execute(&conn)
                .map_err(error::ErrorInternalServerError)
        }))
    })
    .and_then(|_| Ok("uploaded problem statement"))
    .responder()
}

pub fn create_form(req: HttpRequest<AppState>) -> impl Responder {
    render(&req, "admin/problem_statement/create.html", Context::new())
}

pub fn index(req: HttpRequest<AppState>) -> impl Responder {
    req.state()
        .db
        .send(Execute::new(|conn| {
            problem_statements::table
                .inner_join(problems::table)
                .order(problem_statements::id.asc())
                .load::<(ProblemStatement, Problem)>(&conn)
                .map_err(error::ErrorInternalServerError)
        }))
        .from_err()
        .and_then(move |res| match res {
            Ok(problem_statements) => {
                let mut ctx = Context::new();
                ctx.insert("problem_statements", &problem_statements);
                render(&req, "admin/problem_statement/index.html", ctx)
            }
            Err(e) => Err(e),
        })
        .responder()
}
