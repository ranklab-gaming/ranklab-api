// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "media_state"))]
    pub struct MediaState;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "review_state"))]
    pub struct ReviewState;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::MediaState;

    audios (id) {
        created_at -> Timestamp,
        id -> Uuid,
        review_id -> Uuid,
        updated_at -> Timestamp,
        audio_key -> Text,
        processed_audio_key -> Nullable<Text>,
        state -> MediaState,
        transcript -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::MediaState;

    avatars (id) {
        id -> Uuid,
        image_key -> Text,
        processed_image_key -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        state -> MediaState,
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
        password -> Nullable<Text>,
        price -> Int4,
        stripe_account_id -> Text,
        stripe_details_submitted -> Bool,
        stripe_payouts_enabled -> Bool,
        updated_at -> Timestamp,
        emails_enabled -> Bool,
        slug -> Text,
        avatar_id -> Nullable<Uuid>,
        approved -> Bool,
        reviews_count -> Int4,
    }
}

diesel::table! {
    comments (id) {
        body -> Text,
        coach_id -> Uuid,
        created_at -> Timestamp,
        id -> Uuid,
        review_id -> Uuid,
        updated_at -> Timestamp,
        metadata -> Jsonb,
        audio_id -> Nullable<Uuid>,
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
        password -> Nullable<Text>,
        skill_level -> Int2,
        stripe_customer_id -> Text,
        updated_at -> Timestamp,
        emails_enabled -> Bool,
        avatar_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::MediaState;

    recordings (id) {
        created_at -> Timestamp,
        game_id -> Text,
        id -> Uuid,
        player_id -> Uuid,
        skill_level -> Int2,
        title -> Text,
        updated_at -> Timestamp,
        video_key -> Nullable<Text>,
        thumbnail_key -> Nullable<Text>,
        processed_video_key -> Nullable<Text>,
        metadata -> Nullable<Jsonb>,
        state -> MediaState,
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

diesel::joinable!(audios -> reviews (review_id));
diesel::joinable!(coaches -> avatars (avatar_id));
diesel::joinable!(comments -> audios (audio_id));
diesel::joinable!(comments -> coaches (coach_id));
diesel::joinable!(comments -> reviews (review_id));
diesel::joinable!(one_time_tokens -> coaches (coach_id));
diesel::joinable!(one_time_tokens -> players (player_id));
diesel::joinable!(players -> avatars (avatar_id));
diesel::joinable!(recordings -> players (player_id));
diesel::joinable!(reviews -> coaches (coach_id));
diesel::joinable!(reviews -> players (player_id));
diesel::joinable!(reviews -> recordings (recording_id));

diesel::allow_tables_to_appear_in_same_query!(
    audios,
    avatars,
    coaches,
    comments,
    one_time_tokens,
    players,
    recordings,
    reviews,
);
