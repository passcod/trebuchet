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
