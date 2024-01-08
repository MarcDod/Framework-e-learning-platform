-- Your SQL goes here
CREATE TABLE tasks (
    id UUID  DEFAULT uuid_generate_v4() PRIMARY KEY,
    task_doc_id UUID NOT NULL,
    task_package_id UUID NOT NULL,
    task_type VARCHAR(100) NOT NULL,
    state STATE NOT NULL DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT(NOW()) NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT(NOW()) NOT NULL,
    UNIQUE(task_doc_id, task_package_id),
    FOREIGN KEY (task_package_id) REFERENCES task_packages(id)
);