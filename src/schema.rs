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
    }
}

table! {
    games (id) {
        id -> Uuid,
        name -> Text,
    }
}

table! {
    reviews (id) {
        id -> Uuid,
        user_id -> Uuid,
        coach_id -> Nullable<Uuid>,
        title -> Text,
        video_url -> Text,
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
joinable!(reviews -> coaches (coach_id));
joinable!(reviews -> games (game_id));
joinable!(reviews -> users (user_id));

allow_tables_to_appear_in_same_query!(
    coaches,
    comments,
    games,
    reviews,
    users,
);
