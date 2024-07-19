CREATE TABLE match_new(
  id INTEGER PRIMARY KEY NOT NULL,
  user_id INTEGER NOT NULL,
  dog_a_id INTEGER NOT NULL,
  dog_b_id INTEGER NOT NULL,
  status CHAR NOT NULL DEFAULT '…', -- '>', '<', '=', or '…'
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
  FOREIGN KEY(dog_a_id) REFERENCES "dog"(id),
  FOREIGN KEY(dog_b_id) REFERENCES "dog"(id),
  UNIQUE(user_id, dog_a_id, dog_b_id) -- don't want user given same pairing twice; also check at server-level for swapped places i.e. dog b on the left and dog a on the right
);
CREATE TRIGGER update_updated_at_match_new
AFTER UPDATE ON match_new
FOR EACH ROW
BEGIN
  UPDATE match_new SET updated_at = 
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
INSERT INTO match_new SELECT id, user_id, dog_a_id, dog_b_id, status, NULL, NULL from match;
ALTER TABLE match RENAME TO match_old;
ALTER TABLE match_new RENAME TO match;


