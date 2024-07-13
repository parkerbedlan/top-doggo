CREATE TABLE dog (
    id INTEGER PRIMARY KEY NOT NULL,
    image_url TEXT UNIQUE NOT NULL,
    name TEXT UNIQUE NULL,
    namer_id INTEGER NULL,
    FOREIGN KEY (namer_id) REFERENCES "user" (id)
);

CREATE TABLE match (
    id INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL,
    dog_a_id INTEGER NOT NULL,
    dog_b_id INTEGER NOT NULL,
    status CHAR NOT NULL DEFAULT '…', -- '>', '<', '=', or '…'
    FOREIGN KEY (user_id) REFERENCES "user" (id),
    FOREIGN KEY (dog_a_id) REFERENCES "dog" (id),
    FOREIGN KEY (dog_b_id) REFERENCES "dog" (id),
    UNIQUE(user_id, dog_a_id, dog_b_id) -- don't want user given same pairing twice; also check at server-level for swapped places i.e. dog b on the left and dog a on the right
);

CREATE TABLE rating (
    type TEXT NOT NULL DEFAULT 'overall', -- 'overall' or 'personal'
    user_id INTEGER NULL, -- used if type = 'personal'
    dog_id INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES "user" (id),
    FOREIGN KEY (dog_id) REFERENCES "dog" (id),
    PRIMARY KEY(type, user_id, dog_id)
);
