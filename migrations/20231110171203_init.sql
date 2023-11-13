-- Add migration script here
create table if not exists accounts
(
    id         bigserial primary key,
    email      varchar,
    first_name varchar,
    last_name  varchar,
    created_at timestamptz default now(),
    updated_at timestamptz default now()
);

create table if not exists credentials
(
    id              bigserial primary key,
    username        varchar,
    hashed_password varchar,
    user_id         bigint,
    created_at      timestamptz default now(),
    updated_at      timestamptz default now()
)