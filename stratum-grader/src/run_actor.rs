use actix::prelude::*;
use chrono::Utc;
use std::fs;
use std::io::{Error, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use stratum_db::models::{Judgement, Submission, SubmissionFile, TestCase, TestCaseJudgement};

pub struct RunActor {
    pub run_path: PathBuf,
}

impl Actor for RunActor {
    type Context = SyncContext<Self>;
}

pub struct RunTestCaseJudgement {
    test_case: TestCase,
    judgement: Judgement,
    submission: Submission,
    submission_file: SubmissionFile,
}

impl Message for RunTestCaseJudgement {
    type Result = Result<TestCaseJudgement, Error>;
}

impl Handler<RunTestCaseJudgement> for RunActor {
    type Result = Result<TestCaseJudgement, Error>;

    fn handle(&mut self, req: RunTestCaseJudgement, _ctx: &mut Self::Context) -> Self::Result {
        let executable_path = self
            .run_path
            .join("executables")
            .join(req.submission.problem_id.to_string());
        let test_case_path = self.run_path.join("testcases");
        fs::write(
            test_case_path.join(format!(
                "{}.{}.in",
                req.test_case.problem_id, req.test_case.position
            )),
            req.test_case.input,
        )?;
        fs::write(
            test_case_path.join(format!(
                "{}.{}.out",
                req.test_case.problem_id, req.test_case.position
            )),
            req.test_case.output,
        )?;
        let mut cmd = Command::new(&executable_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        {
            let stdin = cmd.stdin.as_mut().expect("Failed to open stdin");
            stdin.write_all(&req.submission_file.content)?;
        }
        let output = cmd.wait_with_output()?;
        Ok(TestCaseJudgement {
            judgement_id: req.judgement.id,
            judgement_grader_id: req.judgement.grader_id,
            test_case_position: req.test_case.position,
            status: output.status.code().unwrap_or(-1), // Happens after signal kill
            output: output.stdout,
            error: output.stderr,
            created_at: Utc::now(),
        })
    }
}
