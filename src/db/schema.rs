table! {
    apps (id) {
        id -> Int4,
        name -> Text,
        created -> Timestamptz,
        updated -> Timestamptz,
        repo -> Text,
        build -> Nullable<Text>,
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

allow_tables_to_appear_in_same_query!(apps, clients,);
