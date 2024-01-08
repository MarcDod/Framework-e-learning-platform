// Documentation was created by ChatGPT
use std::io::{self, Write};

use actix_web::{web::{self, ServiceConfig, Data, Json}, HttpResponse, post};
use uuid::Uuid;

use crate::{repository::{group::GroupRepo, postgres::PgRepo}, jwt, permission, models::groups::{CreateGroupSchema, NewGroup, CreateGroupResponse}};

use super::group_id::group_id;

/// # Create Group Endpoint
///
/// This endpoint creates a new group.
#[utoipa::path(
    post,
    path = "/api/groups/",
    tag = "group",
    request_body = CreateGroupSchema,
    responses(
        (status = 201, description = "The group was successfully created, and information about the new group is provided.", body = CreateGroupResponse),
    ),
)]
#[post("/")]
pub async fn create_group(
    body: Json<CreateGroupSchema>,
    data: Data<GroupRepo>,
    jwt: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    let created_group = match data.create_group(&NewGroup {
        id: Uuid::new_v4(),
        name: body.name.to_string(),
        created_from: jwt.user_id,
        updated_from: jwt.user_id,
        parent: body.parent,
    }) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("{err}");
            io::stdout().flush().unwrap();

            return HttpResponse::InternalServerError().json(
                serde_json::json!({"message": "Something went wrong"})
            )
        }
    };

    HttpResponse::Created().json(CreateGroupResponse {
        id: created_group.id,
        created_at: created_group.created_at.and_utc(),
        name: created_group.name,
        parent: created_group.parent,
    })
}

pub fn config(cfg: &mut ServiceConfig, pgdb: PgRepo) {
    let group_repo = pgdb.new_group_repo();
    cfg.service(
        web::scope("/groups")
            .app_data(Data::<GroupRepo>::new(group_repo.clone()))
            .service(create_group)
            .configure(|cfg| group_id::config(cfg, pgdb.clone()))
    );
}