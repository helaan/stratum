use actix::prelude::*;
use diesel::prelude::*;
use stratum_db::models::{Judgement, Submission};
use stratum_db::schema::{judgements, submissions};

pub struct PollActor {
    pub conn: PgConnection,
    pub grader_id: i32,
}

impl Actor for PollActor {
    type Context = SyncContext<Self>;
}

/**
 *  Polls the submissions that still need to be judged.
 *
 *  Returns: A vector of tuples of submission primary keys and the problem id
 *  it was submitted for.
 */
pub struct SubmissionPoll();

impl Message for SubmissionPoll {
    type Result = Result<Vec<Submission>, diesel::result::Error>;
}

impl Handler<SubmissionPoll> for PollActor {
    type Result = Result<Vec<Submission>, diesel::result::Error>;

    fn handle(&mut self, _: SubmissionPoll, _ctx: &mut Self::Context) -> Self::Result {
        submissions::table
            .select(submissions::all_columns)
            .left_join(
                judgements::table.on(submissions::location_id
                    .eq(judgements::submission_location_id)
                    .and(submissions::id.eq(judgements::submission_id))),
            )
            .filter(judgements::id.is_null())
            .load(&self.conn)
    }
}

pub struct JudgementPoll {
    pub status: i32,
}

impl Message for JudgementPoll {
    type Result = Result<Vec<Judgement>, diesel::result::Error>;
}

impl Handler<JudgementPoll> for PollActor {
    type Result = Result<Vec<Judgement>, diesel::result::Error>;

    fn handle(&mut self, jp: JudgementPoll, _ctx: &mut Self::Context) -> Self::Result {
        judgements::table
            .filter(judgements::grader_id.eq(self.grader_id))
            .filter(judgements::status.eq(jp.status))
            .load(&self.conn)
    }
}
