use crate::poll_actor::PollActor;
use crate::run_actor::RunActor;
use actix::prelude::*;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;
use std::path::PathBuf;

mod poll_actor;
mod run_actor;

fn main() {
    dotenv().ok();
    env_logger::init();
    let system = actix::System::new("stratum-grader");

    let grader_id: i32 = env::var("GRADER_ID")
        .expect("GRADER_ID not set")
        .parse()
        .unwrap();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let poll_addr = SyncArbiter::start(1, move || PollActor {
        conn: PgConnection::establish(&db_url).unwrap(),
        grader_id,
    });

    let run_path = PathBuf::from(env::var("GRADER_RUN_PATH").expect("GRADER_RUN_PATH not set"));
    let run_jobs = env::var("GRADER_RUN_JOBS")
        .expect("GRADER_RUN_JOBS not set")
        .parse()
        .unwrap();
    let run_addr = SyncArbiter::start(run_jobs, move || RunActor {
        run_path: run_path.clone()
    });

    log::debug!("Stratum-grader starting with ID {}", grader_id);

    system.run();
}
