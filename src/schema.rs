table! {
    coaches (id) {
        id -> Uuid,
        user_id -> Uuid,
        name -> Text,
        email -> Text,
        bio -> Text,
        game_id -> Uuid,
    }
}

table! {
    comments (id) {
        id -> Uuid,
        review_id -> Uuid,
        user_id -> Uuid,
        body -> Text,
        video_timestamp -> Int4,
        drawing -> Text,
    }
}

table! {
    games (id) {
        id -> Uuid,
        name -> Text,
    }
}

table! {
    recordings (id) {
        id -> Uuid,
        user_id -> Uuid,
        video_key -> Text,
        upload_url -> Text,
        uploaded -> Bool,
        mime_type -> Text,
    }
}

table! {
    reviews (id) {
        id -> Uuid,
        user_id -> Uuid,
        coach_id -> Nullable<Uuid>,
        title -> Text,
        recording_id -> Uuid,
        game_id -> Uuid,
        notes -> Text,
    }
}

table! {
    users (id) {
        id -> Uuid,
        auth0_id -> Text,
    }
}

joinable!(coaches -> games (game_id));
joinable!(coaches -> users (user_id));
joinable!(comments -> reviews (review_id));
joinable!(comments -> users (user_id));
joinable!(recordings -> users (user_id));
joinable!(reviews -> coaches (coach_id));
joinable!(reviews -> games (game_id));
joinable!(reviews -> recordings (recording_id));
joinable!(reviews -> users (user_id));

allow_tables_to_appear_in_same_query!(
    coaches,
    comments,
    games,
    recordings,
    reviews,
    users,
);
