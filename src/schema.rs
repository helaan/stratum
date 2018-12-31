table! {
    teams (id) {
        id -> Int8,
        name -> Varchar,
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
        rights -> Nullable<Int2>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

joinable!(users -> teams (team_id));

allow_tables_to_appear_in_same_query!(
    teams,
    users,
);
