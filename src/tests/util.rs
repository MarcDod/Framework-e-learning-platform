use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};

use crate::{models::jwt::TokenClaims, AppState};

use crate::models::users::NewUser;
use crate::{
    models::{
        auth::RegisterUserSchema,
        groups::{CreateGroup, NewGroup, NewUserPermission},
        users::UserInfo,
        users::UserResponse,
    },
    repository::users::UsersRepo,
};

use crate::repository::group::GroupRepo;

use crate::{
    models::permissions::{NewRessource, Ressource},
    repository::permissions::PermissionsRepo,
};
use uuid::Uuid;
use crate::models::util::AccessType;
use crate::models::permissions::OptionalUserAccessType;
use crate::models::permissions::PermissionListResponse;
use crate::repository::mongodb::MongoDbRepo;
use crate::models::task::TaskDoc;
use mongodb::bson::doc;
use crate::models::task::TaskResponse;

use mongodb::bson::Document;
use crate::repository::mongodb::SchemaDoc;
use crate::models::roles::NewRole;
use crate::models::roles::UpdateRolePermission;
use crate::models::permissions::PermissionInfo;
use crate::models::{util::TaskPackageType, task::NewTempTask};
use crate::models::task_package::CreatedTaskPackage;
use crate::models::util::Visibility;
use crate::models::solution_attempts::SolutionAttempt;
use crate::schema::solution_attempts::visibility;
use crate::models::members::MemberInfo;
use crate::models::answer::AnswerDoc;

pub fn create_standard_test_user(user_repo: &UsersRepo) -> UserInfo {
    create_test_user(
        &RegisterUserSchema {
            email: "test@test.de".to_string(),
            name: "Test".to_string(),
            password: "123".to_string(),
        },
        user_repo,
    )
}

pub fn create_other_test_user(user_repo: &UsersRepo) -> UserInfo {
    create_test_user(
        &RegisterUserSchema {
            email: "tes@tes.de".to_string(),
            name: "Tes".to_string(),
            password: "123".to_string(),
        },
        user_repo,
    )
}

#[cfg(test)]
pub fn assert_user_response(user_response: &UserResponse, created_user: &UserInfo) {
    assert_eq!(user_response.id, created_user.id);
    assert_eq!(user_response.name, created_user.name);
    assert_eq!(user_response.email, created_user.email);
}

#[cfg(test)]
pub fn assert_permission_response(permission_response: &Vec<PermissionInfo>, user_permission_list: &Vec<(NewUserPermission, Vec<OptionalUserAccessType>)>) {
    for pr in permission_response {
        let perm = user_permission_list.into_iter().find(|up| 
            up.0.group_id == pr.group_id && up.0.ressource == pr.key_value);
        assert!(perm.is_some());
        let value = perm.as_ref().unwrap().clone();
        for at in &pr.access_types {
            let acc = value.clone().1.into_iter().find(|a| a.access_type.to_string() == at.access_type.to_string());
            assert!(acc.is_some());
            let at_value = acc.as_ref().unwrap();
            if (at_value.permission.is_some()) {
                assert_eq!(&at.permission, at_value.permission.as_ref().unwrap());
            }
            if (at_value.set_permission.is_some()) {
                assert_eq!(&at.set_permission, at_value.set_permission.as_ref().unwrap());
            }
            if (at_value.set_set_permission.is_some()) {
                assert_eq!(&at.set_set_permission, at_value.set_set_permission.as_ref().unwrap());
            }
        }
    }
}

#[cfg(test)]
pub fn create_other_permissions(permission_repo: &PermissionsRepo, group_repo: &GroupRepo, ressource: &str, user_id: &Uuid, access_type: &AccessType) {
    let permissions = create_ressource(
        &permission_repo,
        &vec![(NewRessource {
            key_name: &ressource.to_string(),
            key_value: &ressource.to_string(),
        }, vec![access_type.clone(), AccessType::Other])
        ],
    );

    let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
            user_id: user_id.clone(),
            group_id: None,
            ressource: ressource.to_string()
        },
        vec![OptionalUserAccessType {
            access_type: access_type.clone(),
            permission: Some(true),
            set_permission: None,
            set_set_permission: None,
        },OptionalUserAccessType {
            access_type: AccessType::Other,
            permission: Some(true),
            set_permission: None,
            set_set_permission: None,
        }])
    ];

    create_permissions_for_user(
        group_repo,
        &user_permission_list,
    );
}

#[cfg(test)]
pub fn create_permissions(permission_repo: &PermissionsRepo, group_repo: &GroupRepo, ressource: &str, user_id: &Uuid, access_type: &AccessType) {
    let permissions = create_ressource(
        &permission_repo,
        &vec![(NewRessource {
            key_name: &ressource.to_string(),
            key_value: &ressource.to_string(),
        }, vec![access_type.clone()])
        ],
    );

    let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
            user_id: user_id.clone(),
            group_id: None,
            ressource: ressource.to_string()
        },
        vec![OptionalUserAccessType {
            access_type: access_type.clone(),
            permission: Some(true),
            set_permission: None,
            set_set_permission: None,
        }],)
    ];

    create_permissions_for_user(
        group_repo,
        &user_permission_list,
    );
}

#[cfg(test)]
pub fn create_task_package(group_repo: &GroupRepo, tasks: &Vec<NewTempTask>, group_id: &Uuid) -> CreatedTaskPackage {
    group_repo.create_task_package("test", group_id, &None, tasks).unwrap()
}

#[cfg(test)]
pub fn create_solution_attempt(group_repo: &GroupRepo, user_id: &Uuid, task_package_id: &Uuid, group_id: &Uuid) -> SolutionAttempt {
    group_repo.create_solution_attempt(user_id, task_package_id, group_id, &None).unwrap()
}

#[cfg(test)]
pub fn create_private_solution_attempt(group_repo: &GroupRepo, user_id: &Uuid, task_package_id: &Uuid, group_id: &Uuid) -> SolutionAttempt {
    group_repo.create_solution_attempt(user_id, task_package_id, group_id, &Some(Visibility::Private)).unwrap()
}

#[cfg(test)]
pub fn add_member_to_group(group_repo: &GroupRepo, user_id: &Uuid, group_id: &Uuid) -> MemberInfo {
    use crate::models::groups::NewGroupMember;

    group_repo.add_user_to_group(&NewGroupMember {
        group_id,
        user_id
    }).unwrap()
}

#[cfg(test)]
pub fn create_valid_jwt_token(user_id: &Uuid, app_state: &AppState) -> String {
    let now = Utc::now();
    let claims: TokenClaims = TokenClaims {
        sub: user_id.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + Duration::minutes(60)).timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(app_state.env.jwt_secret.as_ref()),
    )
    .unwrap()
}

#[cfg(test)]
pub fn create_invalid_jwt_token(user_id: &Uuid, app_state: &AppState) -> String {
    let now = Utc::now();
    let claims: TokenClaims = TokenClaims {
        sub: user_id.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + Duration::minutes(-20)).timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(app_state.env.jwt_secret.as_ref()),
    )
    .unwrap()
}

#[cfg(test)]
pub fn decode_token_claims(token: &str, app_state: &AppState) -> Option<TokenClaims> {
    use jsonwebtoken::{decode, DecodingKey, Validation};

    match decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(app_state.env.jwt_secret.as_ref()),
        &Validation::default(),
    ) {
        Ok(data) => return Some(data.claims),
        Err(_) => {
            return None;
        }
    }
}

#[cfg(test)]
pub fn validate_token_claims(claims_option: Option<TokenClaims>, user_id: String) {
    assert!(claims_option.is_some(), "No valid token");
    let claims = claims_option.unwrap();
    assert_eq!(claims.sub, user_id, "Token claim has wrong sub")
}

#[cfg(test)]
pub fn create_test_user(new_user: &RegisterUserSchema, user_repo: &UsersRepo) -> UserInfo {
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    use rand_core::OsRng;

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(new_user.password.as_bytes(), &salt)
        .expect("Error while hashing password")
        .to_string();

    user_repo
        .create_user(&NewUser {
            email: &new_user.email.to_lowercase(),
            name: &new_user.name.to_string(),
            password: &hashed_password,
        })
        .unwrap()
}

#[cfg(test)]
pub fn create_groups(group_repo: &GroupRepo, new_groups: Vec<NewGroup>) -> Vec<CreateGroup> {
    let mut groups = vec![];

    for new_group in new_groups {
        groups.push(group_repo.create_group(&new_group).unwrap())
    }

    groups
}

#[cfg(test)]
pub fn create_example_groups(
    group_repo: &GroupRepo,
    amount: usize,
    user_id: Uuid,
) -> Vec<CreateGroup> {

    let mut groups: Vec<NewGroup> = vec![];

    for i in 0..amount {
        let group = NewGroup {
            id: Uuid::new_v4(),
            name: format!("Neue Gruppe {i}"),
            created_from: user_id.clone(),
            updated_from: user_id.clone(),
            parent: None,
        };
        groups.push(group);
    }

    create_groups( 
        group_repo,
        groups,
    )
}

#[cfg(test)]
pub fn create_ressource(
    permission_repo: &PermissionsRepo,
    permission_list: &Vec<(NewRessource, Vec<AccessType>)>,
) -> Vec<(Ressource, Vec<AccessType>)> {
    let mut created_permission_list = vec![];
    for permission in permission_list {
        let ressource = permission_repo.create_ressource(&permission.0).unwrap();
        for access_type in permission.1.clone() {
            permission_repo.add_ressource_access_type(&access_type, &ressource.key_value).unwrap();
        }
        created_permission_list.push((ressource, permission.1.clone()));
    }

    created_permission_list
}

#[cfg(test)]
pub fn create_role(
    permission_repo: &PermissionsRepo,
    role_list: &Vec<NewRole>,
) {
    for role in role_list {
        permission_repo.create_role(role).unwrap();
    }
}

#[cfg(test)]
pub fn update_role(
    permission_repo: &PermissionsRepo,
    update_role: &UpdateRolePermission
) {
    permission_repo.update_permission_on_role(update_role).unwrap();
}

#[cfg(test)]
pub async fn create_task_mc(
    mongodb_repo: &MongoDbRepo
) -> TaskDoc {
    mongodb_repo.create_task("Multiple-Choice".to_string(), doc!{
        "question": "Wie viele Bits ergeben ein Byte?",
        "answers": ["1", "2", "3"]
    }, doc!{
        "solution": 0
    }).await.unwrap()
}

#[cfg(test)]
pub async fn create_correct_answer(
    mongodb_repo: &MongoDbRepo,
    group_repo: &GroupRepo,
    task: &TaskDoc,
    answer_id: &Uuid,
) -> AnswerDoc{
    let answer = mongodb_repo.create_answer_doc(task.clone().solution).await.unwrap();
    let p: usize = group_repo.insert_answer_doc(&answer.id, &answer_id).unwrap();
    assert_eq!(p, 1);
    answer
}

#[cfg(test)]
pub async fn create_incorrect_answer(
    mongodb_repo: &MongoDbRepo,
    group_repo: &GroupRepo,
    answer_id: &Uuid,
) {
    let answer = mongodb_repo.create_answer_doc(doc!{"solution": 15}).await.unwrap();
    let p: usize = group_repo.insert_answer_doc(&answer.id, &answer_id).unwrap();
}

#[cfg(test)]
pub async fn create_task_other(
    mongodb_repo: &MongoDbRepo
) -> TaskDoc {
    mongodb_repo.create_task("other".to_string(), doc!{
        "question": "Wie viele Bits ergeben ein Byte?",
        "answers": ["1", "2", "3"]
    }, doc!{
        "solution": 0
    }).await.unwrap()
}

#[cfg(test)]
pub fn create_example_schema(
) -> (Document, Document) {
    (doc!{
        "type": "object",
        "properties": {
          "question": {
            "type": "string"
          },
          "answers": {
            "type": "array",
            "items": {
              "type": "string"
            }
          }
        },
        "required": [
          "question",
          "answers"
        ],
        "additionalProperties": false
      }, doc!{
        "type": "object",
        "properties": {
          "solution": {
            "type": "integer",
            "minimum": 0
          }
        },
        "required": [
          "solution"
        ],
        "additionalProperties": false
      })
}

#[cfg(test)]
pub fn assert_schma(
 schema_response: &SchemaDoc,
 schema: &SchemaDoc,
) {
    assert_eq!(schema_response.task_type, schema.task_type);
    assert_eq!(schema_response.task_schema, schema.task_schema);
    assert_eq!(schema_response.solution_schema, schema.solution_schema);
}

#[cfg(test)]
pub async fn create_schema(
    mongodb_repo: &MongoDbRepo
) -> SchemaDoc {
    let schema = create_example_schema();

    mongodb_repo.create_schema_doc(schema.0, schema.1, "Multiple-Choice".to_string()).await.unwrap()
}

#[cfg(test)]
pub async fn create_schema_other(
    mongodb_repo: &MongoDbRepo
) -> SchemaDoc {
    let schema = create_example_schema();

    mongodb_repo.create_schema_doc(schema.0, schema.1, "Other".to_string()).await.unwrap()
}


#[cfg(test)]
pub fn create_permissions_for_user(
    group_repo: &GroupRepo,
    permission_list: &Vec<(NewUserPermission, Vec<OptionalUserAccessType>)>,
) {
    for permission in permission_list {
        group_repo.user_set_permission(&permission.0, &permission.1).unwrap();
    }
}

#[cfg(test)]
pub fn password_is_valid(db_password: &String, user_password: &str) -> bool {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let parsed_hash = PasswordHash::new(db_password).unwrap();
    Argon2::default()
        .verify_password(user_password.as_bytes(), &parsed_hash)
        .map_or(false, |_| true)
}

#[cfg(test)]
pub fn assert_task(task_response: &TaskResponse, task: &TaskDoc) {
    assert_eq!(task_response.task, task.task);
    assert_eq!(task_response.id, task.id);
    assert_eq!(task_response.state, task.state);
    assert_eq!(task_response.task_type, task.task_type);
}

#[cfg(test)]
pub fn assert_user(user_repo: &UsersRepo, user_to_assert: &UserResponse, password: &str) {
    use crate::models::util::State;

    let db_user = user_repo.fetch_user(&user_to_assert.id).unwrap();

    assert_eq!(db_user.id, user_to_assert.id, "wrong id");
    assert_eq!(db_user.email, user_to_assert.email, "wrong email");
    assert_eq!(db_user.name, user_to_assert.name, "wrong name");
    assert_eq!(db_user.state, State::Active);
    assert!(
        password_is_valid(&db_user.password, password),
        "wrong password"
    );
    assert_eq!(
        db_user.created_at, db_user.updated_at,
        "created_at != updated_at"
    );
    assert!(
        db_user.created_at.time() < Utc::now().time(),
        "created_at is in the future"
    );
    assert!(
        db_user.updated_at.time() < Utc::now().time(),
        "created_at is in the future"
    );
}

#[cfg(test)]
pub async fn init() -> AppState {
    let app_state = AppState::init().await;

    app_state.pgdb.clear_db();

    app_state
}
