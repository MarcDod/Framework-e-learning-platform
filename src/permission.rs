use std::collections::HashSet;
use std::future::{ready, Ready};
use actix_web::error::{ErrorPreconditionFailed, ErrorForbidden};
use actix_web::web::Data;
use actix_web::{FromRequest, HttpMessage, HttpRequest};
use actix_web::{dev::Payload, Error as ActixWebError};
use uuid::Uuid;

use crate::AppState;
use crate::models::util::{ErrorSchema, AccessType};
use crate::repository::permissions::PermissionsRepo;

pub struct PermissionMiddleware {
    pub permission_addons: Vec<AccessType>,
}

impl FromRequest for PermissionMiddleware {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;

    /// Middleware to check if a user attempting to access a route has the necessary permissions.
    ///
    /// This middleware first checks if the user has global permissions. If not, it then checks if
    /// the user has permissions within a specific group, if the route is associated with a group.
    ///
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let path = req.match_pattern().unwrap();
        let method = req.method();

        let ext = req.extensions();
        let user_id = match ext.get::<Uuid>() {
            Some(v) => v,
            None => return ready(Err(ErrorPreconditionFailed(ErrorSchema {
                message: "No user_id found".to_string(),
            })))
        };

        let app_state = req.app_data::<Data<AppState>>().unwrap().clone();

        if let Some((required_permission, group_pattern, required_addons)) = 
            app_state.permission_config.get_permission_and_group_pattern(&path, method.to_owned()) {
            
            let permission_repo = req.app_data::<Data<PermissionsRepo>>().unwrap().clone();

            if let Some(group_pattern) = group_pattern {
                let group_id: String = req.match_info()
                    .get(group_pattern).unwrap().parse().unwrap();    

                let permission_addons = 
                    permission_repo
                        .user_has_permission(
                            user_id, 
                            required_permission, 
                            &Some(Uuid::parse_str(&group_id).unwrap_or(Uuid::default())),
                        ).unwrap();

                let permission_set: HashSet<AccessType> = permission_addons.iter().copied().collect();
                if  required_addons.iter().all(|required_permission_addon| permission_set.contains(required_permission_addon)) {
                    return ready(Ok(PermissionMiddleware { 
                        permission_addons,
                    }));
                } 
            }
            let permission_addons = 
            permission_repo
                .user_has_permission(
                    user_id, 
                    required_permission,
                    &None,
                ).unwrap();

            let permission_set: HashSet<AccessType> = permission_addons.iter().copied().collect();
            if  required_addons.iter().all(|required_permission_addon| permission_set.contains(required_permission_addon)) {
                return ready(Ok(PermissionMiddleware { 
                    permission_addons,
                }));
            }  else {
                let json_error = ErrorSchema {
                    message: format!("Forbidden access to {} {}", method, path),
                };
                return ready(Err(ErrorForbidden(json_error)));
            }
        }

        return ready(Ok(PermissionMiddleware {
            permission_addons: vec![]
        }));
    }
}