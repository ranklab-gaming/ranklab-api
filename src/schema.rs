table! {
    coaches (id) {
        id -> Uuid,
        name -> Text,
        email -> Text,
        bio -> Text,
        games -> Array<Jsonb>,
        auth0_id -> Text,
    }
}

table! {
    comments (id) {
        id -> Uuid,
        review_id -> Uuid,
        coach_id -> Uuid,
        body -> Text,
        video_timestamp -> Int4,
        drawing -> Text,
    }
}

table! {
    players (id) {
        id -> Uuid,
        auth0_id -> Text,
        name -> Text,
        email -> Text,
        games -> Array<Jsonb>,
    }
}

table! {
    recordings (id) {
        id -> Uuid,
        player_id -> Uuid,
        video_key -> Text,
        upload_url -> Text,
        uploaded -> Bool,
        mime_type -> Text,
    }
}

table! {
    reviews (id) {
        id -> Uuid,
        player_id -> Uuid,
        coach_id -> Nullable<Uuid>,
        title -> Text,
        recording_id -> Uuid,
        game_id -> Text,
        skill_level -> Int2,
        notes -> Text,
        published -> Bool,
    }
}

joinable!(comments -> coaches (coach_id));
joinable!(comments -> reviews (review_id));
joinable!(recordings -> players (player_id));
joinable!(reviews -> coaches (coach_id));
joinable!(reviews -> players (player_id));
joinable!(reviews -> recordings (recording_id));

allow_tables_to_appear_in_same_query!(
    coaches,
    comments,
    players,
    recordings,
    reviews,
);
