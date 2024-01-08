use actix_web::{App, web::{Data, self}, test::{self, TestRequest}, dev::{ServiceResponse, HttpServiceFactory}, http::header::AUTHORIZATION};
use uuid::Uuid;

use crate::{AppState, repository::{users::UsersRepo, group::GroupRepo, permissions::PermissionsRepo}};
use crate::tests::util;


#[derive(Clone)]
#[cfg(test)]
pub struct TestRepo {
    pub app_state: AppState,
    pub user_repo: UsersRepo, 
    pub group_repo: GroupRepo,
    pub permission_repo: PermissionsRepo,
}

#[cfg(test)]
impl TestRepo{
    pub async fn new() -> Self {
        let app_state = util::init().await;
        let user_repo = app_state.pgdb.new_user_repo();
        let group_repo = app_state.pgdb.new_group_repo();
        let permission_repo = app_state.pgdb.new_permissions_repo();

        app_state.pgdb.clear_db();
        app_state.mongodb.clear_db().await;

        TestRepo { app_state, user_repo, group_repo, permission_repo }
    }

    pub fn valid_authorizate(&self, request: TestRequest, user_id: &Uuid) -> TestRequest {
        let valid_access_token = util::create_valid_jwt_token(user_id, &self.app_state);

        request.insert_header((AUTHORIZATION, format!("Bearer {}", valid_access_token)))
    }

    pub fn invalid_authorizate(&self, request: TestRequest, user_id: &Uuid) -> TestRequest {
        let valid_access_token = util::create_invalid_jwt_token(user_id, &self.app_state);

        request.insert_header((AUTHORIZATION, format!("Bearer {}", valid_access_token)))
    }

    pub async fn call<F>(&self, path: &str, scope: &str, factory: F, request: TestRequest) -> ServiceResponse
    where
        F: HttpServiceFactory + 'static,
     {
        let mut app = test::init_service(
            App::new()
                .app_data(Data::new(self.app_state.clone()))
                .app_data(Data::new(self.user_repo.clone()))
                .app_data(Data::new(self.group_repo.clone()))
                .app_data(Data::new(self.permission_repo.clone()))
                .service(web::scope(scope).service(factory)),
        ).await;

        request.uri(path).send_request(&mut app).await
    }
}