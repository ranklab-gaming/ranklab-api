// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "media_state"))]
    pub struct MediaState;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::MediaState;

    audios (id) {
        created_at -> Timestamp,
        id -> Uuid,
        updated_at -> Timestamp,
        audio_key -> Text,
        processed_audio_key -> Nullable<Text>,
        state -> MediaState,
        transcript -> Nullable<Text>,
        user_id -> Uuid,
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
        user_id -> Uuid,
    }
}

diesel::table! {
    comments (id) {
        body -> Text,
        created_at -> Timestamp,
        id -> Uuid,
        updated_at -> Timestamp,
        metadata -> Jsonb,
        audio_id -> Nullable<Uuid>,
        user_id -> Uuid,
        recording_id -> Uuid,
        notified_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    one_time_tokens (id) {
        created_at -> Timestamp,
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        updated_at -> Timestamp,
        used_at -> Nullable<Timestamp>,
        value -> Text,
        scope -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::MediaState;

    recordings (id) {
        created_at -> Timestamp,
        game_id -> Text,
        id -> Uuid,
        user_id -> Uuid,
        skill_level -> Int2,
        title -> Text,
        updated_at -> Timestamp,
        video_key -> Nullable<Text>,
        thumbnail_key -> Nullable<Text>,
        processed_video_key -> Nullable<Text>,
        metadata -> Nullable<Jsonb>,
        state -> MediaState,
        notes -> Text,
    }
}

diesel::table! {
    users (id) {
        created_at -> Timestamp,
        email -> Text,
        game_id -> Text,
        id -> Uuid,
        name -> Text,
        password -> Nullable<Text>,
        updated_at -> Timestamp,
        emails_enabled -> Bool,
        avatar_id -> Nullable<Uuid>,
        skill_level -> Int2,
    }
}

diesel::joinable!(audios -> users (user_id));
diesel::joinable!(comments -> audios (audio_id));
diesel::joinable!(comments -> recordings (recording_id));
diesel::joinable!(comments -> users (user_id));
diesel::joinable!(one_time_tokens -> users (user_id));
diesel::joinable!(recordings -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    audios,
    avatars,
    comments,
    one_time_tokens,
    recordings,
    users,
);
