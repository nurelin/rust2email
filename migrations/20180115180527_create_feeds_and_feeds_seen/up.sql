CREATE TABLE feeds (
	id          INTEGER PRIMARY KEY AUTOINCREMENT,
	name        STRING,
	url         STRING,
	paused      INTEGER,
	last_seen   TIMESTAMP
);
CREATE TABLE feeds_seen (
	id          INTEGER PRIMARY KEY AUTOINCREMENT,
	parent_id   INTEGER,
	url         STRING
);
