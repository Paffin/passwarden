CREATE TABLE web_authn_credentials (
    uuid                  VARCHAR(40)  NOT NULL PRIMARY KEY,
    user_uuid             VARCHAR(40)  NOT NULL REFERENCES users(uuid),
    name                  VARCHAR(255) NOT NULL,
    credential            TEXT         NOT NULL,
    supports_prf          BOOLEAN      NOT NULL DEFAULT FALSE,
    encrypted_user_key    TEXT,
    encrypted_public_key  TEXT,
    encrypted_private_key TEXT
) ENGINE=InnoDB CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

CREATE INDEX idx_web_authn_credentials_user_uuid ON web_authn_credentials(user_uuid);
