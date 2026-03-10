-- Add migration script here

CREATE TABLE files(
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name VARCHAR(100),
  owner UUID NOT NULL REFERENCES users(id),
  url TEXT NOT NULL,
  file_type VARCHAR(100) NOT NULL,
  size BIGINT NOT NULL,
  folder_id UUID REFERENCES folders(id) ON DELETE CASCADE,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
