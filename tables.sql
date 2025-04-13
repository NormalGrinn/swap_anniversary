CREATE TABLE characters (
    character_id INTEGER PRIMARY KEY,
    character_name TEXT UNIQUE,
    character_image TEXT
);

CREATE TABLE users (
    discord_id INTEGER PRIMARY KEY,
    username TEXT UNIQUE,
    character_id INTEGER,
    letter TEXT,
    FOREIGN KEY(character_id) REFERENCES characters(character_id)
);

CREATE TABLE claimed_letters (
    letter_id INTEGER PRIMARY KEY,
    owner_id INTEGER UNIQUE NOT NULL,
    claimee_id INTEGER UNIQUE,
    FOREIGN KEY(owner_id) REFERENCES users(discord_id),
    FOREIGN KEY(claimee_id) REFERENCES users(discord_id)
);