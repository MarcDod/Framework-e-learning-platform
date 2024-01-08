-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TYPE STATE AS ENUM('active', 'deleted'); 

CREATE TABLE users (
    id UUID  DEFAULT uuid_generate_v4() PRIMARY KEY,
    email VARCHAR(100) NOT NULL UNIQUE,
    password VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    state STATE NOT NULL DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT(NOW()) NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT(NOW()) NOT NULL
);