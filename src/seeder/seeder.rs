use uuid::Uuid;

use crate::{AppState, repository::{permissions::PermissionsRepo, users::UsersRepo, group::GroupRepo}, models::{permissions::{NewRessource, Ressource, OptionalUserAccessType}, users::{NewUser, UserInfo}, groups::NewUserPermission, util::AccessType}};

async fn init() -> AppState {
    let app_state = AppState::init().await;

    app_state
}

pub async fn fill_seeder() {
    let app_state = init().await;

    let permissions = create_ressources(app_state.pgdb.new_permissions_repo());
    let users = create_user(app_state.pgdb.new_user_repo()).await;
    create_user_permissions(users[0].id, app_state.pgdb.new_group_repo(), permissions.into_iter().map(|perm| (perm.0.key_value, perm.1)).collect()).await;
}

async fn create_user_permissions(user_id: Uuid, group_repo: GroupRepo, ressource_list: Vec<(String, Vec<AccessType>)>) -> Vec<usize> {
    let mut user_permission_list = vec![];
    for ressource in ressource_list {
        user_permission_list.push(group_repo.user_set_permission(&NewUserPermission {
                group_id: None,
                user_id,
                ressource: ressource.0,
            }, &ressource.1.into_iter().map(|access_type| 
                OptionalUserAccessType {
                    permission: Some(true),
                    set_permission: Some(true),
                    set_set_permission: Some(true),
                    access_type,
                }
            ).collect::<Vec<OptionalUserAccessType>>()).unwrap()
        ); 
    }
    user_permission_list
}

async fn create_user(user_repo: UsersRepo) -> Vec<UserInfo> {
    let user_marc = user_repo.create_user(&NewUser {
        email: &"meine@email.com".to_string(),
        name: &"Admin".to_string(),
        password: &"$argon2id$v=19$m=19456,t=2,p=1$lORjOXTcO0aHXAKm2GFb2g$F3A1GGjVrzp1b859s2Mki+fsuwpMZ64QMtpteNTnw48".to_string(),
    }).unwrap();

    let user_test = user_repo.create_user(&NewUser {
        email: &"test@test.de".to_string(),
        name: &"Test Test".to_string(),
        password: &"$argon2id$v=19$m=19456,t=2,p=1$ADDf4MCz8GWYHMcX/PpWRg$hmz7LUKJwgT5BUIvisVOvpF6Tnj+vQ8wk+u51oDSlbc".to_string(),
    }).unwrap();

    vec![user_marc, user_test]
}

fn create_ressources(permission_repo: PermissionsRepo) -> Vec<(Ressource, Vec<AccessType>)> {
    let perm_1 = permission_repo.create_ressource(&NewRessource {
        key_name: &"Group".to_string(),
        key_value: &"group".to_string(),
    }).unwrap();
    let perm_1_access_types = &vec![AccessType::Create, AccessType::Read, AccessType::Delete];
    for access_type in perm_1_access_types {
        permission_repo.add_ressource_access_type(access_type, &perm_1.key_value).unwrap();
    }

    let perm_2 = permission_repo.create_ressource(&NewRessource {
        key_name: &"Group Meta Data".to_string(),
        key_value: &"group_meta_data".to_string(),
    }).unwrap();
    let perm_2_access_types = &vec![AccessType::Read];
    for access_type in perm_2_access_types {
        permission_repo.add_ressource_access_type(access_type, &perm_2.key_value).unwrap();
    }

    let perm_3 = permission_repo.create_ressource(&NewRessource {
        key_name: &"Group Member".to_string(),
        key_value: &"group_member".to_string(),
    }).unwrap();
    let perm_3_access_types = &vec![AccessType::Read, AccessType::Write];
    for access_type in perm_3_access_types {
        permission_repo.add_ressource_access_type(access_type, &perm_3.key_value).unwrap();
    }

    let perm_4 = permission_repo.create_ressource(&NewRessource {
        key_name: &"Answer".to_string(),
        key_value: &"user_answer".to_string(),
    }).unwrap();
    let perm_4_access_types = &vec![AccessType::Read, AccessType::Write];
    for access_type in perm_4_access_types {
        permission_repo.add_ressource_access_type(access_type, &perm_4.key_value).unwrap();
    }

    let perm_5 = permission_repo.create_ressource(&NewRessource {
        key_name: &"Solution attempt".to_string(),
        key_value: &"solution_attempt".to_string(),
    }).unwrap();
    let perm_5_access_types = &vec![AccessType::Read, AccessType::Create, AccessType::Other];
    for access_type in perm_5_access_types {
        permission_repo.add_ressource_access_type(access_type, &perm_5.key_value).unwrap();
    }

    let perm_6 = permission_repo.create_ressource(&NewRessource {
        key_name: &"Finish of solution attempt".to_string(),
        key_value: &"solution_attempt_finish".to_string(),
    }).unwrap();
    let perm_6_access_types = &vec![AccessType::Write, AccessType::Other];
    for access_type in perm_6_access_types {
        permission_repo.add_ressource_access_type(access_type, &perm_6.key_value).unwrap();
    }

    let perm_7 = permission_repo.create_ressource(&NewRessource {
        key_name: &"Task package".to_string(),
        key_value: &"task_package".to_string(),
    }).unwrap();
    let perm_7_access_types = &vec![AccessType::Read, AccessType::Create, AccessType::Other];
    for access_type in perm_7_access_types {
        permission_repo.add_ressource_access_type(access_type, &perm_7.key_value).unwrap();
    }

    let perm_8 = permission_repo.create_ressource(&NewRessource {
        key_name: &"Task package tasks".to_string(),
        key_value: &"task_package_task".to_string(),
    }).unwrap();
    let perm_8_access_types = &vec![AccessType::Write, AccessType::Read, AccessType::Delete, AccessType::Other];
    for access_type in perm_8_access_types {
        permission_repo.add_ressource_access_type(access_type, &perm_8.key_value).unwrap();
    }

    let perm_9 = permission_repo.create_ressource(&NewRessource {
        key_name: &"Statistic task package".to_string(),
        key_value: &"task_package_statistic".to_string(),
    }).unwrap();
    let perm_9_access_types = &vec![AccessType::Read, AccessType::Other];
    for access_type in perm_9_access_types {
        permission_repo.add_ressource_access_type(access_type, &perm_9.key_value).unwrap();
    }

    let perm_10 = permission_repo.create_ressource(&NewRessource {
        key_name: &"Permission".to_string(),
        key_value: &"permission".to_string(),
    }).unwrap();
    let perm_10_access_types = &vec![AccessType::Read, AccessType::Write, AccessType::Other];
    for access_type in perm_10_access_types {
        permission_repo.add_ressource_access_type(access_type, &perm_10.key_value).unwrap();
    }

    let perm_11 = permission_repo.create_ressource(&NewRessource {
        key_name: &"Tasks".to_string(),
        key_value: &"task".to_string(),
    }).unwrap();
    let perm_11_access_types = &vec![AccessType::Create, AccessType::Delete, AccessType::Read];
    for access_type in perm_11_access_types {
        permission_repo.add_ressource_access_type(access_type, &perm_11.key_value).unwrap();
    }

    let perm_12 = permission_repo.create_ressource(&NewRessource {
        key_name: &"Solution".to_string(),
        key_value: &"solution".to_string(),
    }).unwrap();
    let perm_12_access_types = &vec![AccessType::Read];
    for access_type in perm_12_access_types {
        permission_repo.add_ressource_access_type(access_type, &perm_12.key_value).unwrap();
    }

    let perm_13 = permission_repo.create_ressource(&NewRessource {
        key_name: &"Schemas".to_string(),
        key_value: &"schema".to_string(),
    }).unwrap();
    let perm_13_access_types = &vec![AccessType::Read, AccessType::Create];
    for access_type in perm_13_access_types {
        permission_repo.add_ressource_access_type(access_type, &perm_13.key_value).unwrap();
    }

    vec![
        (perm_1, perm_1_access_types.clone()),
        (perm_2, perm_2_access_types.clone()),
        (perm_3, perm_3_access_types.clone()),
        (perm_4, perm_4_access_types.clone()),
        (perm_5, perm_5_access_types.clone()),
        (perm_6, perm_6_access_types.clone()),
        (perm_7, perm_7_access_types.clone()),
        (perm_8, perm_8_access_types.clone()),
        (perm_9, perm_9_access_types.clone()),
        (perm_10, perm_10_access_types.clone()),
        (perm_11, perm_11_access_types.clone()),
        (perm_12, perm_12_access_types.clone()),
        (perm_13, perm_13_access_types.clone())]
}