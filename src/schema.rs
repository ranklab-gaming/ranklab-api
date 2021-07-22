table! {
    coaches (id) {
        id -> Uuid,
        user_id -> Uuid,
    }
}

table! {
    reviews (id) {
        id -> Uuid,
        user_id -> Uuid,
        coach_id -> Nullable<Uuid>,
        title -> Varchar,
        video_url -> Varchar,
        game -> Varchar,
    }
}

table! {
    users (id) {
        id -> Uuid,
        auth0_id -> Varchar,
    }
}

joinable!(coaches -> users (user_id));
joinable!(reviews -> coaches (coach_id));
joinable!(reviews -> users (user_id));

allow_tables_to_appear_in_same_query!(
    coaches,
    reviews,
    users,
);
