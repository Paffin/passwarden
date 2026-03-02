CREATE TABLE tags (
    uuid        VARCHAR(40)  NOT NULL PRIMARY KEY,
    user_uuid   VARCHAR(40)  NOT NULL REFERENCES users (uuid),
    name        TEXT         NOT NULL,
    created_at  DATETIME     NOT NULL,
    updated_at  DATETIME     NOT NULL
);

CREATE TABLE ciphers_tags (
    cipher_uuid VARCHAR(40) NOT NULL REFERENCES ciphers (uuid),
    tag_uuid    VARCHAR(40) NOT NULL REFERENCES tags (uuid),
    PRIMARY KEY (cipher_uuid, tag_uuid)
);
