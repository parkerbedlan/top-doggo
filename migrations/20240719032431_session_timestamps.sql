CREATE TABLE session_new(
    token TEXT PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT (
      STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW', 'localtime') ||
      ' ' ||
      CASE 
        WHEN STRFTIME('%s', 'now') - STRFTIME('%s', 'now', 'utc') < 0 THEN '-'
        ELSE '+'
      END ||
      SUBSTR('00' || ABS((STRFTIME('%s', 'now') - STRFTIME('%s', 'now', 'utc')) / 3600), -2, 2) ||
      SUBSTR('00' || ABS((STRFTIME('%s', 'now') - STRFTIME('%s', 'now', 'utc')) % 3600 / 60), -2, 2)
    ) NULL,
    updated_at DATETIME DEFAULT (
      STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW', 'localtime') ||
      ' ' ||
      CASE 
        WHEN STRFTIME('%s', 'now') - STRFTIME('%s', 'now', 'utc') < 0 THEN '-'
        ELSE '+'
      END ||
      SUBSTR('00' || ABS((STRFTIME('%s', 'now') - STRFTIME('%s', 'now', 'utc')) / 3600), -2, 2) ||
      SUBSTR('00' || ABS((STRFTIME('%s', 'now') - STRFTIME('%s', 'now', 'utc')) % 3600 / 60), -2, 2)
    ) NULL,
    FOREIGN KEY(user_id) REFERENCES "user"(id)
);
CREATE TRIGGER update_updated_at_session_new
AFTER UPDATE ON session_new
FOR EACH ROW
BEGIN
  UPDATE session_new SET updated_at = 
    STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW', 'localtime') ||
    ' ' ||
    CASE 
      WHEN STRFTIME('%s', 'now') - STRFTIME('%s', 'now', 'utc') < 0 THEN '-'
      ELSE '+'
    END ||
    SUBSTR('00' || ABS((STRFTIME('%s', 'now') - STRFTIME('%s', 'now', 'utc')) / 3600), -2, 2) ||
    SUBSTR('00' || ABS((STRFTIME('%s', 'now') - STRFTIME('%s', 'now', 'utc')) % 3600 / 60), -2, 2)
  WHERE rowid = NEW.rowid;
END;
INSERT INTO session_new (token, user_id, created_at, updated_at) SELECT token, user_id, NULL, NULL from session;
ALTER TABLE session RENAME TO session_old;
ALTER TABLE session_new RENAME TO session;
