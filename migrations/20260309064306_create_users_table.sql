-- Add migration script here

CREATE TYPE gender_type AS ENUM ('Male', 'Female');

CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email VARCHAR(100) UNIQUE NOT NULL,
  name VARCHAR(255),
  avatar TEXT,
  gender gender_type,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
