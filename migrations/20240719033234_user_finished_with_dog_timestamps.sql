CREATE TABLE user_finished_with_dog_new(
  user_id INTEGER NOT NULL,
  dog_id INTEGER NOT NULL,
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
  PRIMARY KEY(user_id, dog_id)
);
CREATE TRIGGER update_updated_at_user_finished_with_dog_new
AFTER UPDATE ON user_finished_with_dog_new
FOR EACH ROW
BEGIN
  UPDATE user_finished_with_dog_new SET updated_at = 
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
INSERT INTO user_finished_with_dog_new SELECT user_id, dog_id, NULL, NULL from user_finished_with_dog;
ALTER TABLE user_finished_with_dog RENAME TO user_finished_with_dog_old;
ALTER TABLE user_finished_with_dog_new RENAME TO user_finished_with_dog;
