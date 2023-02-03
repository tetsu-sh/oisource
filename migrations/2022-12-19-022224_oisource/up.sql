-- Your SQL goes here
create table articles(
    id varchar(100) Primary key,
    title varchar(100) not null,
    author varchar(100) not null,
    media varchar(100) not null,
    url varchar(100) not null, 
    summary varchar(100) not null, 
    created_at datetime not null, 
    crawled_at datetime not null,
    index media_index(media),
    index created_at_index(created_at)
);