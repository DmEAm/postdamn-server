create schema security;
comment on schema security is 'Security level definitions schema';
set search_path to security;

create extension "uuid-ossp";
create extension "ltree";

create domain email as text check (
        octet_length(value) between 6 and 320
        and value like '_%@_%.__%');

comment on domain email is 'Email address';

create domain phone as text check (
        octet_length(value) between 1 + 8 and 1 + 15 + 3
        and value ~ '^\+\d+$');

comment on domain phone is 'Phone number in E.164';

create table users
(
    id    uuid primary key,
    created_at  timestamp default now(),
    name  varchar not null,
    email email   not null,
    phone phone   not null
);

create table roles
(
    id   uuid primary key,
    created_at  timestamp default now(),
    name varchar not null
);

create table permissions
(
    id   ltree primary key,
    created_at  timestamp default now(),
    name varchar not null
);

create table user_roles
(
    user_id uuid references users (id),
    role_id uuid references roles (id),
    primary key (user_id, role_id)
);

create table role_permissions
(
    role_id       uuid references roles (id),
    permission_id ltree references permissions (id),
    primary key (role_id, permission_id)
);

create table user_permissions
(
    user_id       uuid references users (id),
    permission_id ltree references permissions (id),
    primary key (user_id, permission_id)
);

set search_path to public;
