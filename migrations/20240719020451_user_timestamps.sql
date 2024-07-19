CREATE TABLE user_new(
    id INTEGER PRIMARY KEY NOT NULL,
    email TEXT UNIQUE NULL,
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
    ) NULL
);
CREATE TRIGGER update_updated_at_user_new
AFTER UPDATE ON user_new
FOR EACH ROW
BEGIN
  UPDATE user_new SET updated_at = 
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
INSERT INTO user_new (id, email, created_at, updated_at) SELECT id, email, NULL, NULL FROM user;
ALTER TABLE user RENAME TO user_old;
ALTER TABLE user_new RENAME TO user;
