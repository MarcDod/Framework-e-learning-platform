//docu written with chat-gpt
#[cfg(test)]
mod fetch_task_packages_task_info_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::{NewUserPermission, CreateGroupResponse, GroupInfoResponse, GroupMetaDataResponse}, util::{AccessType, PagingSchema, OrderDir}, task::{TaskPagingResponse, NewTempTask, TasksFromPackageResponse}, roles::{NewRole, UpdateRolePermission, NewRolePermission, NewRoleAccessType, UpdateRoleAccesType}, task_package::TaskPackagesResponse}};

    static SCOPE: &'static str = "/api/groups/{group_id}/task_packages/{task_package_id}/tasks"; 
    
    fn get_path(group_id: &Uuid, task_package_id: &Uuid, query: &str) -> String {
        format!("/api/groups/{}/task_packages/{}/tasks/{}", group_id, task_package_id, query)
    }

    /// # Test: `test_fetch_task_packages_task_infos`
    ///
    /// Validates the behavior of the `get_tasks_infos_from_package` handler in Actix-Web when fetching task information from a task package. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Creates a test task (`task`) and adds it to the task package.
    ///    - Assigns the necessary permissions to the user for reading task package tasks.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_tasks_infos_from_package` using the test application instance to fetch task information from the specified task package.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200).
    ///    - Validates the structure and content of the response, including the task information retrieved from the task package.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_task_packages_task_infos() {
        use crate::handlers::groups::group_id::task_packages::task_package_id::tasks::tasks::get_tasks_infos_from_package;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "task_package_task", &created_user.id, &AccessType::Read);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &task_package.id, ""),
            SCOPE,
            get_tasks_infos_from_package,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: TasksFromPackageResponse = test::read_body_json(resp).await;

        assert_eq!(response.tasks.len(), 1);
        assert_eq!(response.tasks[0].id, tasks[0].id);
        assert_eq!(response.tasks[0].task_doc_id, task.id);
        assert_eq!(response.tasks[0].task_package_id, task_package.id);
        assert_eq!(response.tasks[0].task_type, task.task_type);
    }

    /// # Test: `test_fetch_task_packages_task_infos_task_type`
    ///
    /// Validates the behavior of the `get_tasks_infos_from_package` handler in Actix-Web when fetching task information from a task package with a specific task type filter. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Creates two test tasks (`task` and `task_other`) with different task types and adds them to the task package.
    ///    - Assigns the necessary permissions to the user for reading task package tasks.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_tasks_infos_from_package` using the test application instance, specifying a task type filter to fetch task information from the task package.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200).
    ///    - Validates the structure and content of the response, including the task information retrieved from the task package, ensuring that only tasks with the specified task type are included in the response.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_task_packages_task_infos_task_type() {
        use crate::handlers::groups::group_id::task_packages::task_package_id::tasks::tasks::get_tasks_infos_from_package;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "task_package_task", &created_user.id, &AccessType::Read);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;
        let task_other = util::create_task_other(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }, NewTempTask {
            task_doc_id: task_other.id.clone(),
            task_type: task_other.task_type.clone(),
        }]).unwrap();

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &task_package.id, "?task_types[]=Multiple-Choice"),
            SCOPE,
            get_tasks_infos_from_package,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: TasksFromPackageResponse = test::read_body_json(resp).await;

        assert_eq!(response.tasks.len(), 1);
        assert_eq!(response.tasks[0].id, tasks[0].id);
        assert_eq!(response.tasks[0].task_doc_id, task.id);
        assert_eq!(response.tasks[0].task_package_id, task_package.id);
        assert_eq!(response.tasks[0].task_type, task.task_type);
    }

    /// # Test: `test_fetch_task_packages_task_infos_wrong_group`
    ///
    /// Validates the behavior of the `get_tasks_infos_from_package` handler in Actix-Web when attempting to fetch task information from a task package associated with a different group. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Creates a test task (`task`) and adds it to the task package.
    ///    - Assigns the necessary permissions to the user for reading task package tasks.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_tasks_infos_from_package` using the test application instance, specifying a different group ID in the path.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200).
    ///    - Validates the structure and content of the response, ensuring that no tasks are included in the response because the provided group ID in the path does not match the group associated with the task package.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_task_packages_task_infos_wrong_group() {
        use crate::handlers::groups::group_id::task_packages::task_package_id::tasks::tasks::get_tasks_infos_from_package;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "task_package_task", &created_user.id, &AccessType::Read);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let resp = test_app
        .call(
            &get_path(&Uuid::new_v4(), &task_package.id, ""),
            SCOPE,
            get_tasks_infos_from_package,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: TasksFromPackageResponse = test::read_body_json(resp).await;

        assert_eq!(response.tasks.len(), 0);
    }

    /// # Test: `test_fetch_task_packages_task_infos_wrong_task_package`
    ///
    /// Validates the behavior of the `get_tasks_infos_from_package` handler in Actix-Web when attempting to fetch task information from a non-existent task package. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Creates a test task (`task`) and adds it to the task package.
    ///    - Assigns the necessary permissions to the user for reading task package tasks.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_tasks_infos_from_package` using the test application instance, specifying a non-existent task package ID in the path.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200).
    ///    - Validates the structure and content of the response, ensuring that no tasks are included in the response because the provided task package ID in the path does not match any existing task package.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_task_packages_task_infos_wrong_task_package() {
        use crate::handlers::groups::group_id::task_packages::task_package_id::tasks::tasks::get_tasks_infos_from_package;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "task_package_task", &created_user.id, &AccessType::Read);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &Uuid::new_v4(), ""),
            SCOPE,
            get_tasks_infos_from_package,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: TasksFromPackageResponse = test::read_body_json(resp).await;

        assert_eq!(response.tasks.len(), 0);
    }

    /// # Test: `test_fetch_task_packages_task_infos_no_auth`
    ///
    /// Validates the behavior of the `get_tasks_infos_from_package` handler in Actix-Web when attempting to fetch task information without proper authorization. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Creates a test task (`task`) and adds it to the task package.
    ///    - Assigns the necessary permissions to the user for reading task package tasks.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_tasks_infos_from_package` using the test application instance, providing an invalid (unauthorized) authorization token.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "UNAUTHORIZED" (401), indicating that the user does not have the required authorization to fetch task information.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_task_packages_task_infos_no_auth() {
        use crate::handlers::groups::group_id::task_packages::task_package_id::tasks::tasks::get_tasks_infos_from_package;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "task_package_task", &created_user.id, &AccessType::Read);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &task_package.id, ""),
            SCOPE,
            get_tasks_infos_from_package,
            test_app.invalid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
    }

    /// # Test: `test_fetch_task_packages_task_infos_no_permission`
    ///
    /// Validates the behavior of the `get_tasks_infos_from_package` handler in Actix-Web when attempting to fetch task information without the necessary permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Creates a test task (`task`) and adds it to the task package.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_tasks_infos_from_package` using the test application instance, providing a valid authorization token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating that the user does not have the required permissions to fetch task information.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_task_packages_task_infos_no_permission() {
        use crate::handlers::groups::group_id::task_packages::task_package_id::tasks::tasks::get_tasks_infos_from_package;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &task_package.id, ""),
            SCOPE,
            get_tasks_infos_from_package,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    }
}