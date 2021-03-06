use chrono::Utc;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;
use std::fs;
use std::io::{Error, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str;
use std::thread::sleep;
use std::time::Duration;
use stratum_db::judgement_status::JudgementStatus;
use stratum_db::models::{Judgement, Submission, SubmissionFile, TestCase, TestCaseJudgement};
use stratum_db::schema::{
    judgements, submission_files, submissions, test_case_judgements, test_cases,
};

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

    let grader_id: i32 = env::var("GRADER_ID")
        .expect("GRADER_ID not set")
        .parse()
        .unwrap();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let conn = PgConnection::establish(&db_url).unwrap();

    let run_path = PathBuf::from(env::var("GRADER_RUN_PATH").expect("GRADER_RUN_PATH not set"));

    log::info!("Stratum-grader starting with ID {}", grader_id);

    let mut idle = false;

    loop {
        // Create judgements
        let subs = submissions::table
            .select(submissions::all_columns)
            .left_join(
                judgements::table.on(submissions::location_id
                    .eq(judgements::submission_location_id)
                    .and(submissions::id.eq(judgements::submission_id))
                    .and(judgements::grader_id.eq(grader_id))
                    .and(judgements::valid.eq(true))),
            )
            .filter(judgements::id.is_null())
            .load::<Submission>(&conn)
            .unwrap();
        if !idle {
            log::info!("Checked for new submissions, {} found", subs.len());
        }

        if subs.is_empty() {
            let one_sec = Duration::from_secs(1);
            sleep(one_sec);
        }
        idle = subs.is_empty();

        for sub in subs {
            log::info!(
                "Queuing judgement for {},{} on grader {}",
                sub.id,
                sub.location_id,
                grader_id
            );
            diesel::insert_into(judgements::table)
                .values((
                    judgements::grader_id.eq(grader_id),
                    judgements::submission_id.eq(sub.id),
                    judgements::submission_location_id.eq(sub.location_id),
                    judgements::status.eq(JudgementStatus::Queued as i32),
                    judgements::valid.eq(true),
                ))
                //.on_conflict_do_nothing()
                .execute(&conn)
                .unwrap();
        }

        // Judge something!
        conn.transaction::<_, failure::Error, _>(|| {
            let judgement_opt = judgements::table
                .filter(judgements::grader_id.eq(grader_id))
                .filter(judgements::status.eq(JudgementStatus::Queued as i32))
                .for_update()
                .skip_locked()
                .first::<Judgement>(&conn)
                .optional()?;

            if judgement_opt.is_none() {
                return Ok(());
            }
            idle = false;

            let mut judgement = judgement_opt.unwrap();

            let submission = submissions::table
                .find((judgement.submission_location_id, judgement.submission_id))
                .first::<Submission>(&conn)?;

            log::info!(
                "Judging {} for submission {},{}...",
                judgement.id,
                submission.id,
                submission.location_id
            );

            let test_cases = test_cases::table
                .filter(test_cases::problem_id.eq(submission.problem_id))
                .load::<TestCase>(&conn)?;

            let submission_file = submission_files::table
                .find((
                    submission.location_id,
                    submission.id,
                    &submission.entry_point,
                ))
                .first::<SubmissionFile>(&conn)?;

            let test_case_judgements = test_cases.iter().map(|test_case| {
                run(
                    &run_path,
                    &test_case,
                    &judgement,
                    &submission,
                    &submission_file,
                )
                // If Judging crashed for some reason: log the crash and move
                // this submission out of the way.
                .unwrap_or_else(|err| {
                    log::error!("Error occurred when grading {:?}", err);
                    TestCaseJudgement {
                        judgement_id: judgement.id,
                        judgement_grader_id: judgement.grader_id,
                        test_case_position: test_case.position,
                        status_code: 1, // Internal error
                        output: format!("{:?}", err).into_bytes(),
                        error: Vec::new(),
                        created_at: Utc::now(),
                    }
                })
            });
            let mut score = 0;
            let mut success = true;
            let mut judgement_status = JudgementStatus::Accepted;
            for tcj in test_case_judgements {
                if tcj.status_code != 42 {
                    success = false;
                    judgement_status = if tcj.status_code == 43 {
                        JudgementStatus::WrongAnswer
                    } else {
                        JudgementStatus::JudgingError
                    };
                    score = 0;
                }
                if success {
                    score += str::from_utf8(&tcj.output)?.trim().parse::<i64>()?;
                }
                tcj.insert_into(test_case_judgements::table)
                    .execute(&conn)?;
                if !success {
                    break;
                }
            }
            judgement.status = judgement_status as i32;
            judgement.score = if success { Some(score) } else { None };
            judgement.save_changes::<Judgement>(&conn)?;
            Ok(())
        })
        .unwrap();
    }
}

fn run(
    run_path: &Path,
    test_case: &TestCase,
    judgement: &Judgement,
    submission: &Submission,
    submission_file: &SubmissionFile,
) -> Result<TestCaseJudgement, Error> {
    let executable_path = run_path
        .join("executables")
        .join(format!("{}.{}", submission.problem_id, test_case.position));
    let test_case_path = run_path.join("testcases");
    let input_path = test_case_path.join(format!(
        "{}.{}.in",
        test_case.problem_id, test_case.position
    ));
    let output_path = test_case_path.join(format!(
        "{}.{}.out",
        test_case.problem_id, test_case.position
    ));
    fs::write(&input_path, &test_case.input)?;
    fs::write(&output_path, &test_case.output)?;
    log::info!(
        "Running {} {} {}...",
        executable_path.display(),
        input_path.display(),
        output_path.display()
    );
    let mut cmd = Command::new(&executable_path)
        .arg(input_path)
        //.arg(output_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    {
        let stdin = cmd.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(&submission_file.content)?;
        // According to POSIX all files end on a newline, so if ours doesn't, add one.
        if submission_file.content.last() != Some(&b'\n') {
            stdin.write_all(b"\n")?;
        }
    }
    let output = cmd.wait_with_output()?;
    Ok(TestCaseJudgement {
        judgement_id: judgement.id,
        judgement_grader_id: judgement.grader_id,
        test_case_position: test_case.position,
        status_code: output.status.code().unwrap_or(-1), // Happens after signal kill
        output: output.stdout,
        error: output.stderr,
        created_at: Utc::now(),
    })
}
