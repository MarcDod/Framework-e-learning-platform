-- Your SQL goes here
CREATE TYPE ANSWER_STATE AS ENUM('active', 'deleted', 'done');

CREATE TABLE answers (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    solution_attempt_id UUID NOT NULL,
    answer_doc_id UUID NOT NULL,
    task_id UUID NOT NULL,
    created_from UUID NOT NULL,
    correct BOOLEAN DEFAULT(FALSE) NOT NULL,
    state ANSWER_STATE NOT NULL DEFAULT 'done',
    UNIQUE(task_id, solution_attempt_id, answer_doc_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id),
    FOREIGN KEY (solution_attempt_id) REFERENCES solution_attempts(id),
    FOREIGN KEY (created_from) REFERENCES users(id)
);