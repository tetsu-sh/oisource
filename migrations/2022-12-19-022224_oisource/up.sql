-- Your SQL goes here
create table articles(
    id varchar(100) Primary key,
    title varchar(100),
    auther varchar(100),
    media varchar(100) not null,
    url varchar(100) not null, 
    summary varchar(100), 
    created_at datetime not null, 
    crawled_at datetime not null
);