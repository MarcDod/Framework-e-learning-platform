// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "access_type"))]
    pub struct AccessType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "answer_state"))]
    pub struct AnswerState;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "state"))]
    pub struct State;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "task_package_type"))]
    pub struct TaskPackageType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "visibility"))]
    pub struct Visibility;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AnswerState;

    answers (id) {
        id -> Uuid,
        solution_attempt_id -> Uuid,
        answer_doc_id -> Uuid,
        task_id -> Uuid,
        created_from -> Uuid,
        correct -> Bool,
        state -> AnswerState,
    }
}

diesel::table! {
    group_ancestors (group_id, ancestor_group_id) {
        group_id -> Uuid,
        ancestor_group_id -> Uuid,
    }
}

diesel::table! {
    group_members (id) {
        id -> Uuid,
        group_id -> Uuid,
        user_id -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::State;

    groups (id) {
        id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        state -> State,
        parent -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_from -> Uuid,
        updated_from -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AccessType;

    ressource_access_types (ressource, access_type) {
        #[max_length = 45]
        ressource -> Varchar,
        access_type -> AccessType,
    }
}

diesel::table! {
    ressources (key_value) {
        #[max_length = 45]
        key_name -> Varchar,
        #[max_length = 45]
        key_value -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AccessType;

    role_access_types (role_permission_id, access_type) {
        role_permission_id -> Uuid,
        access_type -> AccessType,
        permission -> Bool,
        set_permission -> Bool,
        set_set_permission -> Bool,
    }
}

diesel::table! {
    role_permissions (id) {
        id -> Uuid,
        #[max_length = 45]
        role -> Varchar,
        #[max_length = 45]
        ressource -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::State;

    roles (value_key) {
        #[max_length = 45]
        name -> Varchar,
        state -> State,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 45]
        value_key -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Visibility;

    solution_attempts (id) {
        id -> Uuid,
        user_id -> Uuid,
        task_package_id -> Uuid,
        visibility -> Visibility,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::State;
    use super::sql_types::TaskPackageType;

    task_packages (id) {
        id -> Uuid,
        group_id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        state -> State,
        task_package_type -> TaskPackageType,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::State;

    tasks (id) {
        id -> Uuid,
        task_doc_id -> Uuid,
        task_package_id -> Uuid,
        #[max_length = 100]
        task_type -> Varchar,
        state -> State,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AccessType;

    user_access_types (user_permission_id, access_type) {
        user_permission_id -> Uuid,
        access_type -> AccessType,
        permission -> Bool,
        set_permission -> Bool,
        set_set_permission -> Bool,
    }
}

diesel::table! {
    user_permissions (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 45]
        ressource -> Varchar,
        group_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::State;

    users (id) {
        id -> Uuid,
        #[max_length = 100]
        email -> Varchar,
        #[max_length = 100]
        password -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        state -> State,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(answers -> solution_attempts (solution_attempt_id));
diesel::joinable!(answers -> tasks (task_id));
diesel::joinable!(answers -> users (created_from));
diesel::joinable!(group_members -> groups (group_id));
diesel::joinable!(group_members -> users (user_id));
diesel::joinable!(ressource_access_types -> ressources (ressource));
diesel::joinable!(role_access_types -> role_permissions (role_permission_id));
diesel::joinable!(role_permissions -> ressources (ressource));
diesel::joinable!(role_permissions -> roles (role));
diesel::joinable!(solution_attempts -> task_packages (task_package_id));
diesel::joinable!(solution_attempts -> users (user_id));
diesel::joinable!(task_packages -> groups (group_id));
diesel::joinable!(tasks -> task_packages (task_package_id));
diesel::joinable!(user_access_types -> user_permissions (user_permission_id));
diesel::joinable!(user_permissions -> groups (group_id));
diesel::joinable!(user_permissions -> ressources (ressource));
diesel::joinable!(user_permissions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    answers,
    group_ancestors,
    group_members,
    groups,
    ressource_access_types,
    ressources,
    role_access_types,
    role_permissions,
    roles,
    solution_attempts,
    task_packages,
    tasks,
    user_access_types,
    user_permissions,
    users,
);
