-- Add migration script here
alter table accounts
    add column created_at_epoch int;
alter table accounts
    add column updated_at_epoch int;
update accounts
set created_at_epoch = extract(epoch from created_at),
    updated_at_epoch = extract(epoch from updated_at);

alter table accounts
    rename created_at to created_at_old;
alter table accounts
    rename updated_at to updated_at_old;
alter table accounts
    rename created_at_epoch to created_at;
alter table accounts
    rename updated_at_epoch to updated_at;

alter table accounts drop column created_at_old;
alter table accounts drop column updated_at_old;
alter table accounts alter column created_at set default extract(epoch from now());
alter table accounts alter column updated_at set default extract(epoch from now());


alter table credentials
    add column created_at_epoch int;
alter table credentials
    add column updated_at_epoch int;
update credentials
set created_at_epoch = extract(epoch from created_at),
    updated_at_epoch = extract(epoch from updated_at);
alter table credentials
    rename created_at to created_at_old;
alter table credentials
    rename updated_at to updated_at_old;
alter table credentials
    rename created_at_epoch to created_at;
alter table credentials
    rename updated_at_epoch to updated_at;
alter table credentials drop column created_at_old;
alter table credentials drop column updated_at_old;
alter table credentials alter column created_at set default extract(epoch from now());
alter table credentials alter column updated_at set default extract(epoch from now());

