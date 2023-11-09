create table "bind_code"
(
    "id" SERIAL PRIMARY KEY,
    "email" VARCHAR(100) NOT NULL,
    "account" CHAR(42) NOT NULL,
    "code" CHAR(6) NOT NULL,
    "status" SMALLINT NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    "updated_at" TIMESTAMPTZ
);
