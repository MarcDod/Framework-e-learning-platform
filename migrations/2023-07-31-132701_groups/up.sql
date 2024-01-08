-- Your SQL goes here
CREATE TABLE groups (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    state STATE NOT NULL DEFAULT 'active',
    parent UUID,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT(NOW()) NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT(NOW()) NOT NULL,
    created_from UUID NOT NULL,
    updated_from UUID NOT NULL,
    FOREIGN KEY (created_from) REFERENCES users(id),
    FOREIGN KEY (updated_from) REFERENCES users(id),
    FOREIGN KEY (parent) REFERENCES groups(id)
);