// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "recording_state"))]
    pub struct RecordingState;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "review_state"))]
    pub struct ReviewState;
}

diesel::table! {
    coach_invitations (id) {
        created_at -> Timestamp,
        id -> Uuid,
        updated_at -> Timestamp,
        used_at -> Nullable<Timestamp>,
        value -> Text,
    }
}

diesel::table! {
    coaches (id) {
        bio -> Text,
        country -> Text,
        created_at -> Timestamp,
        email -> Text,
        game_id -> Text,
        id -> Uuid,
        name -> Text,
        password -> Text,
        price -> Int4,
        stripe_account_id -> Text,
        stripe_details_submitted -> Bool,
        stripe_payouts_enabled -> Bool,
        updated_at -> Timestamp,
        emails_enabled -> Bool,
        slug -> Text,
    }
}

diesel::table! {
    comments (id) {
        body -> Text,
        coach_id -> Uuid,
        created_at -> Timestamp,
        drawing -> Text,
        id -> Uuid,
        review_id -> Uuid,
        updated_at -> Timestamp,
        metadata -> Jsonb,
    }
}

diesel::table! {
    one_time_tokens (id) {
        coach_id -> Nullable<Uuid>,
        created_at -> Timestamp,
        id -> Uuid,
        player_id -> Nullable<Uuid>,
        updated_at -> Timestamp,
        used_at -> Nullable<Timestamp>,
        value -> Text,
    }
}

diesel::table! {
    players (id) {
        created_at -> Timestamp,
        email -> Text,
        game_id -> Text,
        id -> Uuid,
        name -> Text,
        password -> Text,
        skill_level -> Int2,
        stripe_customer_id -> Text,
        updated_at -> Timestamp,
        emails_enabled -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RecordingState;

    recordings (id) {
        created_at -> Timestamp,
        game_id -> Text,
        id -> Uuid,
        player_id -> Uuid,
        skill_level -> Int2,
        title -> Text,
        updated_at -> Timestamp,
        video_key -> Nullable<Text>,
        state -> RecordingState,
        thumbnail_key -> Nullable<Text>,
        processed_video_key -> Nullable<Text>,
        metadata -> Nullable<Jsonb>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ReviewState;

    reviews (id) {
        coach_id -> Uuid,
        created_at -> Timestamp,
        id -> Uuid,
        notes -> Text,
        player_id -> Uuid,
        recording_id -> Uuid,
        state -> ReviewState,
        stripe_payment_intent_id -> Text,
        updated_at -> Timestamp,
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
    coach_invitations,
    coaches,
    comments,
    one_time_tokens,
    players,
    recordings,
    reviews,
);
