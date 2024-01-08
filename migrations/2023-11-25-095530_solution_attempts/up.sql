-- Your SQL goes here
CREATE TYPE VISIBILITY AS ENUM('private', 'public');  


CREATE TABLE solution_attempts (
    id UUID  DEFAULT uuid_generate_v4() PRIMARY KEY,
    user_id UUID NOT NULL,
    task_package_id UUID NOT NULL,
    visibility VISIBILITY NOT NULL DEFAULT 'public',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT(NOW()) NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT(NOW()) NOT NULL,
    FOREIGN KEY (task_package_id) REFERENCES task_packages(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);