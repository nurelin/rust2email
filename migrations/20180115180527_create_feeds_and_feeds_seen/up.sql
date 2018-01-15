CREATE TABLE IF NOT EXISTS feeds (
	id          INTEGER PRIMARY KEY AUTOINCREMENT,
	name        STRING,
	url         STRING,
	paused      INTEGER DEFAULT 0,
	last_seen   INTEGER DEFAULT 0
)
CREATE TABLE IF NOT EXISTS feeds_seen (
	id          INTEGER PRIMARY KEY AUTOINCREMENT,
	parent_id   INTEGER,
	url         STRING
)
