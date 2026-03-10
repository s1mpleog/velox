-- Add migration script here

CREATE TYPE token_type AS ENUM ('signin', 'login');

CREATE TABLE magic_tokens (
  token text primary key,
  email varchar(100) unique not null,
  kind token_type,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  expires_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP + INTERVAL '5 minutes'
);
