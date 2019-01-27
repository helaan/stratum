table! {
    contest_problems (contest_id, problem_id) {
        contest_id -> Int8,
        problem_id -> Int8,
        label -> Varchar,
    }
}

table! {
    contests (id) {
        id -> Int8,
        name -> Varchar,
        short_name -> Varchar,
        start_at -> Nullable<Timestamptz>,
        freeze_at -> Nullable<Timestamptz>,
        end_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    problems (id) {
        id -> Int8,
        name -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    problem_statements (id) {
        id -> Int8,
        problem_id -> Int8,
        filename -> Varchar,
        mimetype -> Varchar,
        statement -> Bytea,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    sessions (key) {
        key -> Uuid,
        user_id -> Int8,
        created_at -> Timestamptz,
    }
}

table! {
    submission_files (submission_location_id, submission_id, filename) {
        submission_id -> Int8,
        submission_location_id -> Int4,
        filename -> Varchar,
        mimetype -> Varchar,
        content -> Bytea,
    }
}

table! {
    submissions (location_id, id) {
        id -> Int8,
        location_id -> Int4,
        problem_id -> Int8,
        team_id -> Int8,
        entry_point -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    teams (id) {
        id -> Int8,
        name -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    test_cases (problem_id, position) {
        problem_id -> Int8,
        position -> Int4,
        description -> Varchar,
        input -> Bytea,
        input_mimetype -> Varchar,
        output -> Bytea,
        output_mimetype -> Varchar,
        visible_rights -> Int2,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    users (id) {
        id -> Int8,
        team_id -> Nullable<Int8>,
        username -> Varchar,
        password_hash -> Varchar,
        rights -> Int2,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

joinable!(contest_problems -> contests (contest_id));
joinable!(contest_problems -> problems (problem_id));
joinable!(problem_statements -> problems (problem_id));
joinable!(sessions -> users (user_id));
joinable!(submissions -> problems (problem_id));
joinable!(submissions -> teams (team_id));
joinable!(test_cases -> problems (problem_id));
joinable!(users -> teams (team_id));

allow_tables_to_appear_in_same_query!(
    contest_problems,
    contests,
    problems,
    problem_statements,
    sessions,
    submission_files,
    submissions,
    teams,
    test_cases,
    users,
);
