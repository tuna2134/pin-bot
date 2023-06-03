-- Add migration script here
CREATE TABLE Message(
    ChannelId BIGINT,
    MessageContent BIGINT,
    AuthorId BIGINT
)