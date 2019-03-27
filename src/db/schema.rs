table! {
    apps (id) {
        id -> Int4,
        name -> Text,
        created -> Timestamptz,
        updated -> Timestamptz,
        repo -> Text,
        build_script -> Text,
    }
}

table! {
    clients (id) {
        id -> Int4,
        connection -> Uuid,
        connected -> Bool,
        created -> Timestamptz,
        updated -> Timestamptz,
        target -> Bool,
        app -> Text,
        name -> Text,
        tags -> Nullable<Array<Text>>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::Release_state;
    releases (id) {
        id -> Int4,
        app_id -> Nullable<Int4>,
        tag -> Text,
        created -> Timestamptz,
        updated -> Timestamptz,
        repo -> Text,
        build_script -> Text,
        state -> Release_state,
    }
}

joinable!(releases -> apps (app_id));

allow_tables_to_appear_in_same_query!(apps, clients, releases,);
