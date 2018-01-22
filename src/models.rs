use schema::feeds;
use schema::feeds_seen;
use chrono::NaiveDateTime;

#[derive(Identifiable)]
#[derive(Queryable)]
#[table_name="feeds"]
pub struct Feeds {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub paused: bool,
    pub last_seen: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name="feeds"]
pub struct NewFeed<'a> {
    pub name: &'a str,
    pub url: &'a str,
    pub paused: bool,
    pub last_seen: NaiveDateTime,
}

#[derive(Queryable)]
#[derive(Identifiable)]
#[table_name="feeds_seen"]
pub struct FeedsSeen {
    pub id: i32,
    pub parent_id: i32,
    pub url: String,
}

#[derive(Insertable)]
#[table_name="feeds_seen"]
pub struct NewFeedSeen<'a> {
    pub parent_id: i32,
    pub url: &'a str,
}
