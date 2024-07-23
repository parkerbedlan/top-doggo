-- manually delete former _old tables first
-- DROP TABLE user_old;
-- DROP TABLE session_old;
-- DROP TABLE dog_old;
-- DROP TABLE match_old;
-- DROP TABLE rating_old;
-- DROP TABLE user_finished_with_dog_old;

CREATE TABLE user_new(
    id INTEGER PRIMARY KEY NOT NULL,
    created_at DATETIME NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NULL DEFAULT CURRENT_TIMESTAMP,
    email TEXT UNIQUE NULL
);
CREATE TRIGGER update_updated_at_user_simple
AFTER UPDATE ON user_new
WHEN OLD.updated_at <> CURRENT_TIMESTAMP
BEGIN
    UPDATE user_new
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = OLD.id;
END;
INSERT INTO user_new (id, email, created_at, updated_at) SELECT id, email, NULL, NULL FROM user;
ALTER TABLE user RENAME TO user_old2;
ALTER TABLE user_new RENAME TO user;

CREATE TABLE session_new(
    created_at DATETIME NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NULL DEFAULT CURRENT_TIMESTAMP,
    token TEXT PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL,
    FOREIGN KEY(user_id) REFERENCES "user"(id)
);
CREATE TRIGGER update_updated_at_session_simple
AFTER UPDATE ON session_new
WHEN OLD.updated_at <> CURRENT_TIMESTAMP
BEGIN
    UPDATE session_new
    SET updated_at = CURRENT_TIMESTAMP
    WHERE token = OLD.token;
END;
INSERT INTO session_new (token, user_id, created_at, updated_at) SELECT token, user_id, NULL, NULL from session;
ALTER TABLE session RENAME TO session_old2;
ALTER TABLE session_new RENAME TO session;

CREATE TABLE dog_new(
    id INTEGER PRIMARY KEY NOT NULL,
    created_at DATETIME NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NULL DEFAULT CURRENT_TIMESTAMP,
    image_url TEXT UNIQUE NOT NULL,
    name TEXT UNIQUE NULL,
    namer_id INTEGER NULL,
    approved BOOLEAN DEFAULT TRUE NOT NULL,
    FOREIGN KEY(namer_id) REFERENCES "user"(id)
);
CREATE TRIGGER update_updated_at_dog_simple
AFTER UPDATE ON dog_new
WHEN OLD.updated_at <> CURRENT_TIMESTAMP
BEGIN
    UPDATE dog_new
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = OLD.id;
END;
INSERT INTO dog_new SELECT id, NULL, NULL, image_url, name, namer_id, approved from dog;
ALTER TABLE dog RENAME TO dog_old2;
ALTER TABLE dog_new RENAME TO dog;

CREATE TABLE match_new(
    id INTEGER PRIMARY KEY NOT NULL,
    created_at DATETIME NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NULL DEFAULT CURRENT_TIMESTAMP,
    user_id INTEGER NOT NULL,
    dog_a_id INTEGER NOT NULL,
    dog_b_id INTEGER NOT NULL,
    status CHAR NOT NULL DEFAULT '…', -- '>', '<', '=', or '…'
    elo_change_overall_a INTEGER NULL,
    elo_change_overall_b INTEGER NULL,
    elo_change_personal_a INTEGER NULL,
    elo_change_personal_b INTEGER NULL,
    FOREIGN KEY(user_id) REFERENCES "user"(id),
    FOREIGN KEY(dog_a_id) REFERENCES "dog"(id),
    FOREIGN KEY(dog_b_id) REFERENCES "dog"(id),
    UNIQUE(user_id, dog_a_id, dog_b_id) -- don't want user given same pairing twice; also check at server-level for swapped places i.e. dog b on the left and dog a on the right
);
CREATE TRIGGER update_updated_at_match_simple
AFTER UPDATE ON match_new
WHEN OLD.updated_at <> CURRENT_TIMESTAMP
BEGIN
    UPDATE match_new
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = OLD.id;
END;
INSERT INTO match_new SELECT id, NULL, NULL, user_id, dog_a_id, dog_b_id, status, elo_change_overall_a, elo_change_overall_b, elo_change_personal_a, elo_change_personal_b from match;
ALTER TABLE match RENAME TO match_old2;
ALTER TABLE match_new RENAME TO match;

CREATE TABLE rating_new(
    created_at DATETIME NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NULL DEFAULT CURRENT_TIMESTAMP,
    type TEXT NOT NULL DEFAULT 'overall', -- 'overall' or 'personal'
    user_id INTEGER NULL, -- used if type = 'personal'
    dog_id INTEGER NOT NULL,
    value INTEGER NOT NULL DEFAULT 1000,
    FOREIGN KEY(user_id) REFERENCES "user"(id),
    FOREIGN KEY(dog_id) REFERENCES "dog"(id),
    PRIMARY KEY(type, user_id, dog_id)
);
CREATE TRIGGER update_updated_at_rating_simple
AFTER UPDATE ON rating_new
WHEN OLD.updated_at <> CURRENT_TIMESTAMP
BEGIN
    UPDATE rating_new
    SET updated_at = CURRENT_TIMESTAMP
    WHERE type = OLD.type AND user_id = OLD.user_id AND dog_id = OLD.dog_id;
END;
INSERT INTO rating_new SELECT NULL, NULL, type, user_id, dog_id, value from rating;
ALTER TABLE rating RENAME TO rating_old2;
ALTER TABLE rating_new RENAME TO rating;

CREATE TABLE user_finished_with_dog_new(
    created_at DATETIME NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NULL DEFAULT CURRENT_TIMESTAMP,
    user_id INTEGER NOT NULL,
    dog_id INTEGER NOT NULL,
    FOREIGN KEY(user_id) REFERENCES "user"(id),
    FOREIGN KEY(dog_id) REFERENCES "dog"(id),
    PRIMARY KEY(user_id, dog_id)
);
CREATE TRIGGER update_updated_at_user_finished_with_dog_simple
AFTER UPDATE ON user_finished_with_dog_new
WHEN OLD.updated_at <> CURRENT_TIMESTAMP
BEGIN
    UPDATE user_finished_with_dog_new
    SET updated_at = CURRENT_TIMESTAMP
    WHERE user_id = OLD.user_id AND dog_id = OLD.dog_id;
END;
INSERT INTO user_finished_with_dog_new SELECT NULL, NULL, user_id, dog_id from user_finished_with_dog;
ALTER TABLE user_finished_with_dog RENAME TO user_finished_with_dog_old2;
ALTER TABLE user_finished_with_dog_new RENAME TO user_finished_with_dog;

CREATE TABLE log_new(
    id INTEGER PRIMARY KEY NOT NULL,
    created_at DATETIME NULL DEFAULT CURRENT_TIMESTAMP,
    action TEXT NOT NULL,
    user_id INTEGER NULL,
    client_ip TEXT NULL,
    notes TEXT NULL,
    FOREIGN KEY(user_id) REFERENCES user(id)
);
INSERT INTO log_new SELECT id, NULL, action, user_id, client_ip, notes FROM log;
ALTER TABLE log RENAME TO log_old2;
ALTER TABLE log_new RENAME TO log;
