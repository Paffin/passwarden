CREATE TABLE tags (
    uuid        TEXT    NOT NULL PRIMARY KEY,
    user_uuid   TEXT    NOT NULL REFERENCES users (uuid),
    name        TEXT    NOT NULL,
    created_at  DATETIME NOT NULL,
    updated_at  DATETIME NOT NULL
);

CREATE TABLE ciphers_tags (
    cipher_uuid TEXT NOT NULL REFERENCES ciphers (uuid),
    tag_uuid    TEXT NOT NULL REFERENCES tags (uuid),
    PRIMARY KEY (cipher_uuid, tag_uuid)
);
