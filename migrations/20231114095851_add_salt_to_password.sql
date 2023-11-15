-- Add migration script here
alter table credentials add salt varchar;
create unique index credentials_username_uindex
    on credentials (username);
create unique index accounts_email_uindex
    on accounts (email);
