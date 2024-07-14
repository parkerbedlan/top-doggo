CREATE TABLE user_finished_with_dog (
    user_id INTEGER NOT NULL,
    dog_id INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES "user" (id),
    FOREIGN KEY (dog_id) REFERENCES "dog" (id),
    PRIMARY KEY(user_id, dog_id)
);
