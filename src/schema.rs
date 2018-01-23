table!{
    feeds {
        id -> Integer,
        name -> Text,
        url -> Text,
        paused -> Bool,
        last_seen -> Integer,
    }
}
table!{
    feeds_seen {
        id -> Integer,
        parent_id -> Integer,
        url -> Text,
    }
}
