-- Add migration script here
create table if not exists assessments
(
    id          bigserial primary key,
    user_id     bigint,
    started_at  int,
    finished_at int,
    created_at  int,
    updated_at  int
);

create table if not exists assessment_questions
(
    id            bigserial primary key,
    question_id   bigint,
    assessment_id bigint,
    serial_number int,
    answer_at     int,

    created_at    int,
    updated_at    int
);

create table if not exists questions
(
    id         bigserial primary key,
    content    json,
    answer     json,

    created_at int,
    updated_at int
);
