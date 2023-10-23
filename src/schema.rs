// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "media_state"))]
    pub struct MediaState;
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
        user_id -> Uuid,
        recording_id -> Uuid,
        notified_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    followings (id) {
        id -> Uuid,
        user_id -> Uuid,
        game_id -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
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
        state -> MediaState,
        notes -> Text,
    }
}

diesel::table! {
    users (id) {
        created_at -> Timestamp,
        email -> Text,
        id -> Uuid,
        name -> Text,
        password -> Nullable<Text>,
        updated_at -> Timestamp,
        emails_enabled -> Bool,
        avatar_id -> Nullable<Uuid>,
        digest_notified_at -> Timestamp,
    }
}

diesel::joinable!(comments -> recordings (recording_id));
diesel::joinable!(comments -> users (user_id));
diesel::joinable!(followings -> users (user_id));
diesel::joinable!(one_time_tokens -> users (user_id));
diesel::joinable!(recordings -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    avatars,
    comments,
    followings,
    one_time_tokens,
    recordings,
    users,
);
