create table crate
(
    id           int auto_increment
        primary key,
    name         varchar(64) charset utf8 not null,
    version      varchar(40) charset utf8 not null,
    dependencies int                      not null,
    constraint crate_name_version_uindex
        unique (name, version)
);

create table crate_dependency
(
    id       int auto_increment
        primary key,
    crate_id int                      null,
    name     varchar(64) charset utf8 not null,
    version  varchar(40) charset utf8 not null,
    constraint crate_dependency_name_version_crate_id_uindex
        unique (name, version, crate_id),
    constraint crate_dependency_crate_id_fk
        foreign key (crate_id) references crate (id)
            on delete cascade
);

