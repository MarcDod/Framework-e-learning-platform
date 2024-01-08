use super::{user_docs, auth_docs, groups_docs, member_docs, task_docs, task_packages_docs, solution_attempts_docs, answer_docs, permission_docs};

pub struct ApiDoc;


impl utoipa::OpenApi for ApiDoc {
    fn openapi() -> utoipa::openapi::OpenApi {
        let mut open_api = user_docs::ApiDoc::openapi();
        open_api.merge(auth_docs::ApiDoc::openapi());
        open_api.merge(groups_docs::ApiDoc::openapi());
        open_api.merge(member_docs::ApiDoc::openapi());
        open_api.merge(task_docs::ApiDoc::openapi());
        open_api.merge(task_packages_docs::ApiDoc::openapi());
        open_api.merge(solution_attempts_docs::ApiDoc::openapi());
        open_api.merge(answer_docs::ApiDoc::openapi());
        open_api.merge(permission_docs::ApiDoc::openapi());

        open_api
    }
}

