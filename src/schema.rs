table! {
    coaches (id) {
        id -> Uuid,
        user_id -> Uuid,
        name -> Text,
        email -> Text,
        bio -> Text,
        game -> Text,
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
    reviews (id) {
        id -> Uuid,
        user_id -> Uuid,
        coach_id -> Nullable<Uuid>,
        title -> Text,
        video_url -> Text,
        game -> Text,
    }
}

table! {
    users (id) {
        id -> Uuid,
        auth0_id -> Text,
    }
}

joinable!(coaches -> users (user_id));
joinable!(comments -> reviews (review_id));
joinable!(comments -> users (user_id));
joinable!(reviews -> coaches (coach_id));
joinable!(reviews -> users (user_id));

allow_tables_to_appear_in_same_query!(coaches, comments, reviews, users,);
