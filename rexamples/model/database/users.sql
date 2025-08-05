create table logos."user"
(
    id          bigint primary key          not null,
    application integer                     not null,
    category    bigint                      not null default 0, -- 第一分类
    family      bigint                      not null default 0, -- 第二分类
    properties  jsonb                       not null default '{}'::jsonb,
    information json                        not null default '{}'::json,
    marks       json                        not null default '{}'::json,
    gender      smallint                    not null default '-1'::integer,
    state       integer                     not null,
    joined      timestamp without time zone not null,
    updated     timestamp without time zone not null
);
comment on column logos."user".category is '第一分类';
comment on column logos."user".family is '第二分类';

create table logos.user_auth_log
(
    id       bigint primary key not null default nextval('user_auth_log_id_seq'::regclass),
    uid      bigint             not null,
    action   integer            not null,
    property jsonb              not null default '{}'::jsonb,
    context  jsonb              not null default '{}'::jsonb
);

create table logos.user_bind
(
    id        bigint primary key          not null default nextval('user_bind_id_seq'::regclass),
    uid       bigint                      not null,
    platform  character varying(255)      not null,
    proof     jsonb                       not null default '{}'::jsonb,
    expire_at timestamp without time zone not null,
    device    jsonb                       not null default '{}'::jsonb,
    state     integer                     not null,
    created   timestamp without time zone not null,
    updated   timestamp without time zone not null
);

