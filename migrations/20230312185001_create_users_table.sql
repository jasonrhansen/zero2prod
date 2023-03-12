CREATE TABLE
  users (
    id uuid NOT NULL,
    PRIMARY KEY (id),
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL
  );
