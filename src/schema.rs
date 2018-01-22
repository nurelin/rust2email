table!{
    feeds {
        id -> Integer,
        name -> Text,
        url -> Text,
        paused -> Bool,
        last_seen -> Timestamp,
    }
}
table!{
    feeds_seen {
        id -> Integer,
        parent_id -> Integer,
        url -> Text,
    }
}
