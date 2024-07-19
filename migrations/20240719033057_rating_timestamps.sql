CREATE TABLE rating_new(
  type TEXT NOT NULL DEFAULT 'overall', -- 'overall' or 'personal'
  user_id INTEGER NULL, -- used if type = 'personal'
  dog_id INTEGER NOT NULL,
  value INTEGER NOT NULL DEFAULT 1000,
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
  FOREIGN KEY(user_id) REFERENCES "user"(id),
  FOREIGN KEY(dog_id) REFERENCES "dog"(id),
  PRIMARY KEY(type, user_id, dog_id)
);
CREATE TRIGGER update_updated_at_rating_new
AFTER UPDATE ON rating_new
FOR EACH ROW
BEGIN
  UPDATE rating_new SET updated_at = 
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
INSERT INTO rating_new SELECT type, user_id, dog_id, value, NULL, NULL from rating;
ALTER TABLE rating RENAME TO rating_old;
ALTER TABLE rating_new RENAME TO rating;


