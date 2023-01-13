// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "review_state"))]
    pub struct ReviewState;
}

diesel::table! {
    coaches (id) {
        email -> Text,
        name -> Text,
        bio -> Text,
        country -> Text,
        game_ids -> Array<Nullable<Text>>,
        id -> Uuid,
        password -> Text,
        stripe_account_id -> Nullable<Text>,
        stripe_details_submitted -> Bool,
        stripe_payouts_enabled -> Bool,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    comments (id) {
        body -> Text,
        coach_id -> Uuid,
        drawing -> Text,
        id -> Uuid,
        review_id -> Uuid,
        video_timestamp -> Int4,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    one_time_tokens (id) {
        id -> Uuid,
        value -> Text,
        player_id -> Nullable<Uuid>,
        coach_id -> Nullable<Uuid>,
        scope -> Text,
        used_at -> Nullable<Timestamp>,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    players (id) {
        email -> Text,
        name -> Text,
        games -> Array<Nullable<Jsonb>>,
        id -> Uuid,
        stripe_customer_id -> Nullable<Text>,
        password -> Text,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    recordings (id) {
        id -> Uuid,
        mime_type -> Text,
        player_id -> Uuid,
        upload_url -> Text,
        uploaded -> Bool,
        video_key -> Text,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ReviewState;

    reviews (id) {
        coach_id -> Uuid,
        game_id -> Text,
        id -> Uuid,
        notes -> Text,
        player_id -> Uuid,
        recording_id -> Uuid,
        skill_level -> Int2,
        title -> Text,
        state -> ReviewState,
        stripe_order_id -> Text,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::joinable!(comments -> coaches (coach_id));
diesel::joinable!(comments -> reviews (review_id));
diesel::joinable!(one_time_tokens -> coaches (coach_id));
diesel::joinable!(one_time_tokens -> players (player_id));
diesel::joinable!(recordings -> players (player_id));
diesel::joinable!(reviews -> coaches (coach_id));
diesel::joinable!(reviews -> players (player_id));
diesel::joinable!(reviews -> recordings (recording_id));

diesel::allow_tables_to_appear_in_same_query!(
    coaches,
    comments,
    one_time_tokens,
    players,
    recordings,
    reviews,
);
