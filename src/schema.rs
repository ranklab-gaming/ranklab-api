table! {
    coaches (id) {
        auth0_id -> Text,
        bio -> Text,
        country -> Text,
        email -> Text,
        games -> Array<Jsonb>,
        id -> Uuid,
        name -> Text,
        stripe_account_id -> Nullable<Text>,
        stripe_details_submitted -> Bool,
        stripe_payouts_enabled -> Bool,
    }
}

table! {
    comments (id) {
        body -> Text,
        coach_id -> Uuid,
        drawing -> Text,
        id -> Uuid,
        review_id -> Uuid,
        video_timestamp -> Int4,
    }
}

table! {
    players (id) {
        auth0_id -> Text,
        email -> Text,
        games -> Array<Jsonb>,
        id -> Uuid,
        name -> Text,
        stripe_customer_id -> Nullable<Text>,
    }
}

table! {
    recordings (id) {
        id -> Uuid,
        mime_type -> Text,
        player_id -> Uuid,
        upload_url -> Text,
        uploaded -> Bool,
        video_key -> Text,
    }
}

table! {
    review_intents (id) {
        game_id -> Text,
        id -> Uuid,
        notes -> Text,
        player_id -> Uuid,
        recording_id -> Nullable<Uuid>,
        review_id -> Nullable<Uuid>,
        stripe_payment_intent_id -> Text,
        title -> Text,
    }
}

table! {
    reviews (id) {
        coach_id -> Nullable<Uuid>,
        game_id -> Text,
        id -> Uuid,
        notes -> Text,
        player_id -> Uuid,
        published -> Bool,
        recording_id -> Uuid,
        skill_level -> Int2,
        title -> Text,
    }
}

joinable!(comments -> coaches (coach_id));
joinable!(comments -> reviews (review_id));
joinable!(recordings -> players (player_id));
joinable!(review_intents -> players (player_id));
joinable!(review_intents -> recordings (recording_id));
joinable!(review_intents -> reviews (review_id));
joinable!(reviews -> coaches (coach_id));
joinable!(reviews -> players (player_id));
joinable!(reviews -> recordings (recording_id));

allow_tables_to_appear_in_same_query!(
    coaches,
    comments,
    players,
    recordings,
    review_intents,
    reviews,
);
