-- Your SQL goes here
CREATE TYPE TASK_PACKAGE_TYPE AS ENUM('learning', 'exam');  

CREATE TABLE task_packages (
    id UUID  DEFAULT uuid_generate_v4() PRIMARY KEY,
    group_id UUID NOT NULL,
    name VARCHAR(100) NOT NULL,
    state STATE NOT NULL DEFAULT 'active',
    task_package_type TASK_PACKAGE_TYPE NOT NULL DEFAULT 'learning',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT(NOW()) NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT(NOW()) NOT NULL,
    UNIQUE(name, group_id),
    FOREIGN KEY (group_id) REFERENCES groups(id)
);