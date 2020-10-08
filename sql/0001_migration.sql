create table `rust-kata-001`.crate_deps
(
    id                 int auto_increment
        primary key,
    name               varchar(64) charset utf8 not null,
    version            varchar(40) charset utf8 not null,
    dependency_name    varchar(64) charset utf8 not null,
    dependency_version varchar(40) charset utf8 not null,
    constraint crate_deps_name_version_dependency_name_uindex
        unique (name, version, dependency_name)
);

create index crate_deps_dependency_name_dependency_version_index
    on `rust-kata-001`.crate_deps (dependency_name, dependency_version);
