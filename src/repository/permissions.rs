// Documentation was created by ChatGPT
use diesel::{QueryDsl, IntoSql, prelude::*, ExpressionMethods, dsl::{exists, count_star}, RunQueryDsl, result::Error, BoolExpressionMethods, sql_types::Bool, BoxableExpression, Connection, helper_types::LeftJoin};
use uuid::Uuid;

use crate::{models::{groups::NewUserPermission, permissions::{Ressource, PermissionInfoListWithCount, RessourceListWithCount, NewRessource, PermissionInfo, OptionalUserAccessType, UserAccessType, RessourceAndAccessTypesListWithCount, RessourceWithAccessTypes}, util::{PagingSchema, AccessType}, roles::{UpdateRolePermission, NewRole, NewRoleAccessType}}, repository::group};

use super::postgres::DBPool;

#[derive(Clone)]
pub struct PermissionsRepo {
    pool: DBPool
}

impl PermissionsRepo {

    pub fn new(pool: DBPool) -> Self {
        PermissionsRepo { pool }
    }

    /// Creates a new resource and retrieves its details.
    ///
    /// This function inserts a new resource into the 'ressources' table based on the provided
    /// `NewRessource` struct. After insertion, it fetches and returns the newly created resource's details.
    /// The operation is performed within a database transaction to ensure atomicity.
    ///
    /// # Parameters
    /// - `&self`: Reference to the current instance of the struct containing this method.
    /// - `new_ressource`: Reference to a `NewRessource` struct containing the details of the resource to be created.
    ///
    /// # Returns
    /// `Result<Ressource, Error>`: On success, returns a `Ressource` struct representing the newly created resource.
    /// On failure, returns an `Error`.
    ///
    /// # Errors
    /// This function might return an error if there are issues with database connectivity, transaction execution,
    /// data insertion, or retrieval of the new resource.
    ///
    /// # Example
    /// ```
    /// let new_resource = NewRessource { /* fields */ };
    /// let result = instance.create_ressource(&new_resource);
    /// match result {
    ///     Ok(resource) => println!("Created new resource: {:?}", resource),
    ///     Err(e) => println!("Error creating resource: {}", e),
    /// }
    /// ```
    pub fn create_ressource(
        &self,
        new_ressource: &NewRessource,
    ) -> Result<Ressource, Error> {
        use crate::schema::ressources;

        let conn = 
            &mut self.pool.get().unwrap();

        conn.transaction(|conn| {
            diesel::insert_into(ressources::table)
                .values(new_ressource)
                .execute(conn)?;

                ressources::table
                .select((ressources::key_value, ressources::key_name))
                .filter(ressources::key_value.eq(new_ressource.key_value))
                .first(conn)
        })
    }

    pub fn add_ressource_access_type(
        &self,
        access_type: &AccessType,
        ressource: &String
    ) -> Result<usize, Error> {
        use crate::schema::ressource_access_types;

        let conn = 
            &mut self.pool.get().unwrap();

        diesel::insert_into(ressource_access_types::table)
            .values((
                ressource_access_types::access_type.eq(access_type),
                ressource_access_types::ressource.eq(ressource),
            ))
            .execute(conn)
    }


    fn need_set_set_permission(
        &self,
        user_permission_addon: &OptionalUserAccessType
    ) -> bool {
        user_permission_addon.set_permission.is_some() || user_permission_addon.set_set_permission.is_some()
    }

    pub fn user_has_permission(
        &self, 
        user_id: &Uuid, 
        ressource: &String,
        group_id: &Option<Uuid>,
    ) -> Result<Vec<AccessType>, Error> {
        use crate::schema::ressources;
        use crate::schema::user_permissions;
        use crate::schema::user_access_types;

        let conn = 
            &mut self.pool.get().unwrap();

        let mut filter_query: Box<dyn BoxableExpression<_, diesel::pg::Pg, SqlType = Bool>> = 
            Box::new(
                user_permissions::user_id.eq(user_id)
                .and(ressources::key_value.eq(ressource))
            );

        let check_global_permission = user_permissions::group_id.is_null();

        if group_id.is_some() {
            let group_id_value = group_id.unwrap_or_default();
            filter_query = Box::new(
                filter_query.and(
                    check_global_permission
                    .or(
                        user_permissions::group_id.eq(group_id_value).is_not_null()
                    )
                )
            );
        } else {
            filter_query = Box::new(filter_query.and(check_global_permission));
        }

        user_permissions::table
            .inner_join(ressources::table)
            .inner_join(user_access_types::table)
            .select(user_access_types::access_type)
            .filter(
                filter_query
                .and(user_access_types::permission.eq(true))
                
            ).load(conn)
    }

    pub fn user_can_set_permission(
        &self,
        new_permission: &NewUserPermission,
        access_types: &OptionalUserAccessType,
        user_id: &Uuid,
    ) -> Result<bool, Error> {
        use crate::schema::user_permissions;
        use crate::schema::user_access_types;

        let conn 
        = &mut self.pool.get().unwrap();

        let mut filter_query: Box<dyn BoxableExpression<_, diesel::pg::Pg, SqlType = Bool>> = Box::new(
            user_permissions::user_id.eq(user_id)
            .and(user_permissions::ressource.eq(&new_permission.ressource))
        );

        if self.need_set_set_permission(access_types) {
            filter_query = Box::new(filter_query.and(user_access_types::set_set_permission.eq(true)));
        }

        if access_types.permission.is_some() {
            filter_query = Box::new(filter_query.and(user_access_types::set_permission.eq(true)));
        }

        let check_global_permission = user_permissions::group_id.is_null();

        if new_permission.group_id.is_some() {
            let group_id_value = new_permission.group_id.unwrap_or_default();
            filter_query = Box::new(
                filter_query.and(
                    check_global_permission
                    .or(
                        user_permissions::group_id.eq(group_id_value).is_not_null()
                    )
                )
            );
        } else {
            filter_query = Box::new(filter_query.and(check_global_permission));
        }

        let sub_query = user_permissions::table
        .inner_join(
            user_access_types::table
        )
        .select(0.into_sql::<diesel::sql_types::Integer>())
        .filter(filter_query);


        diesel::select(exists(sub_query)).get_result::<bool>(conn)
    }

    pub fn user_can_set_permissions(
        &self,
        new_permission: &NewUserPermission,
        access_types: &Vec<OptionalUserAccessType>,
        user_id: &Uuid,
    ) -> Result<bool, Error> {
        let mut empty = 0;
        for permission_addon in access_types {
            if permission_addon.permission.is_none() 
                && permission_addon.set_permission.is_none() 
                && permission_addon.set_set_permission.is_none() {
                empty += 1;
                continue;
            }
            let can = match self.user_can_set_permission(new_permission, permission_addon, user_id) {
                Ok(can) => can,
                Err(err) => return Err(err)
            };
            if can == false {
                return Ok(false);
            }
        }
        Ok(empty != access_types.len())
    }

    /// Fetches a list of permissions based on specified criteria.
    ///
    /// This function retrieves a list of permissions from the database, considering optional criteria such as
    /// pagination and filtering by a list of permission values. The result includes a count of total permissions
    /// that match the criteria.
    ///
    /// # Arguments
    ///
    /// * `pagination` - A reference to a `PagingSchema` struct specifying the pagination details, such as page number and limit.
    /// * `permission_value_list` - An optional list of permission values to filter the permissions. If `None`, no filtering is applied.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `PermissionListWithCount` struct, which includes the following information:
    ///
    /// 1. `Vec<Permission>` - A list of `Permission` structs, each representing a permission with the following details:
    ///    - `Uuid` - The unique identifier of the permission.
    ///    - `i32` - The key value associated with the permission.
    ///    - `String` - The key name associated with the permission.
    /// 2. `usize` - The total count of permissions that match the specified criteria.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with the database connection or if an error occurs while querying for permissions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use diesel::prelude::*;
    ///
    /// let db_conn = PgRepo::establish_connection("database_url");
    /// let permission_repo = db_conn.new_permissions_repo();
    /// let pagination = PagingSchema { page: 1, limit: 10 }; // Replace with the desired pagination details.
    /// let permission_values = Some(vec![123, 456]); // Replace with the desired permission values or use None for no filtering.
    ///
    /// match permission_repo.fetch_permissions(&pagination, &permission_values) {
    ///     Ok(permission_list) => {
    ///         println!("Total permissions: {}", permission_list.total_count);
    ///         for permission in permission_list.permission_list {
    ///             println!("Permission ID: {}, Key Value: {}, Key Name: {}",
    ///                 permission.id, permission.key_value, permission.key_name);
    ///         }
    ///     },
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    ///
    /// In this example, the function is used to fetch a list of permissions based on the given criteria and print the result.
    pub fn fetch_ressources(
        &self,
        pagination: &PagingSchema,
        ressource_list: &Option<Vec<String>>,
    ) -> Result<RessourceListWithCount, Error> {
        use crate::schema::ressources;

        let conn 
        = &mut self.pool.get().unwrap();

        let offset = pagination.page * pagination.limit;

        let mut filter_query: Box<dyn BoxableExpression<_, diesel::pg::Pg, SqlType = Bool>> = Box::new(
            ressources::key_value.is_not_null()
        );

        conn.transaction(|conn| {
            
            let total_count = match ressources::table
                .select(count_star())
                .first(conn) {
                    Ok(v) => v,
                    Err(err) => return Err(err),
                };

            if ressource_list.is_some() {
                filter_query = Box::new(
                    filter_query
                    .and(ressources::key_value.eq_any(ressource_list.as_ref().unwrap()))
                );
            }

            let ressources = match ressources::table
                .select((
                    ressources::key_value, ressources::key_name
                ))
                .filter(filter_query)
                .limit(pagination.limit.into())
                .offset(offset.into())
                .load(conn) {
                    Ok(v) => v,
                    Err(err) => return Err(err),
                };

            Ok(RessourceListWithCount { 
                ressources, 
                total_count, 
            })
        })
    }

    pub fn fetch_ressources_with_access_type(
        &self,
        pagination: &PagingSchema,
        ressource_list: &Option<Vec<String>>,
    ) -> Result<RessourceAndAccessTypesListWithCount, Error> {
        use crate::schema::ressources;
        use crate::schema::ressource_access_types;

        let conn 
        = &mut self.pool.get().unwrap();

        let offset = pagination.page * pagination.limit;

        let mut filter_query: Box<dyn BoxableExpression<_, diesel::pg::Pg, SqlType = Bool>> = Box::new(
            ressources::key_value.is_not_null()
        );

        conn.transaction(|conn| {
            
            let total_count = match ressources::table
                .select(count_star())
                .first(conn) {
                    Ok(v) => v,
                    Err(err) => return Err(err),
                };

            if ressource_list.is_some() {
                filter_query = Box::new(
                    filter_query
                    .and(ressources::key_value.eq_any(ressource_list.as_ref().unwrap()))
                );
            }

            let ressources: Vec<Ressource> = match ressources::table
                .select((
                    ressources::key_value, ressources::key_name
                ))
                .filter(filter_query)
                .limit(pagination.limit.into())
                .offset(offset.into())
                .load(conn) {
                    Ok(v) => v,
                    Err(err) => return Err(err),
                };

            let mut ressources_with_access_types: Vec<RessourceWithAccessTypes> = vec![];

            for ressource in ressources {
                let access_types: Vec<AccessType> = match ressource_access_types::table
                    .select(
                        ressource_access_types::access_type
                    )
                    .filter(ressource_access_types::ressource.eq(&ressource.key_value))
                    .load(conn) {
                        Ok(v) => v,
                        Err(err) => return Err(err),
                    };
                
                ressources_with_access_types.push(RessourceWithAccessTypes {
                    access_types,
                    key_name: ressource.key_name,
                    key_value: ressource.key_value,
                })
            }

            Ok(RessourceAndAccessTypesListWithCount {
                ressources: ressources_with_access_types,
                total_count
            })
        })
    }

    pub fn fetch_user_group_only_permissions(
        &self,
        user_id: &Uuid,
        pagination: &PagingSchema,
        group_id: &Uuid,
        ressource_list: &Vec<String>,
    ) -> Result<PermissionInfoListWithCount, Error> {
        use crate::schema::user_permissions;
        use crate::schema::ressources;
        use crate::schema::user_access_types;

        let conn 
        = &mut self.pool.get().unwrap();

        let mut filter_query: Box<dyn BoxableExpression<_, diesel::pg::Pg, SqlType = Bool>> = Box::new(
            user_permissions::user_id.eq(user_id)
            .and(
                user_permissions::group_id.eq(group_id).is_not_null()
            )
        );

        let offset = pagination.page * pagination.limit;

        conn.transaction(|conn| {
            
            let total_count = match user_permissions::table
                .inner_join(ressources::table)
                .select(count_star())
                .filter(&filter_query)
                .first(conn) {
                    Ok(v) => v,
                    Err(err) => return Err(err),
                };

            filter_query = Box::new(
                filter_query
                .and(ressources::key_value.eq_any(ressource_list))
            );

            let user_permission_list: Vec<(Uuid, String, String, Option<Uuid>)> = match user_permissions::table
                .inner_join(ressources::table)
                .select((user_permissions::id,
                    ressources::key_value, ressources::key_name, user_permissions::group_id
                ))
                .filter(filter_query)
                .limit(pagination.limit.into())
                .offset(offset.into())
                .load(conn) {
                    Ok(v) => v,
                    Err(err) => return Err(err),
                };

            let mut permission_list: Vec<PermissionInfo> = vec![];

            for user_permission in user_permission_list {
                let addon_list = user_access_types::table
                    .select((
                        user_access_types::access_type,
                        user_access_types::set_permission,
                        user_access_types::set_set_permission,   
                        user_access_types::permission,
                    )).filter(
                        user_access_types::user_permission_id.eq(user_permission.0)
                    )
                    .load(conn).unwrap();

                    permission_list.push(
                    PermissionInfo {
                        key_value: user_permission.1, 
                        key_name: user_permission.2, 
                        access_types: addon_list,
                        group_id: user_permission.3,
                    }
                );
            }

            Ok(PermissionInfoListWithCount { 
                permission_list, 
                total_count, 
            })
        })
    }

    /// Fetches user permissions with optional filtering and pagination.
    ///
    /// This function retrieves a list of user permissions from the database, optionally filtered by group ID,
    /// resource list, and a flag indicating whether to include only group-specific permissions. It supports
    /// pagination through `PagingSchema`. The result includes both the list of permissions and the total count
    /// of permissions matching the filter criteria.
    ///
    /// # Parameters
    /// - `&self`: Reference to the current instance of the struct containing this method.
    /// - `user_id`: Reference to a UUID representing the ID of the user whose permissions are to be fetched.
    /// - `pagination`: Reference to a `PagingSchema` struct for pagination details.
    /// - `group_id`: Optional reference to a UUID representing the group ID for filtering permissions.
    /// - `only_group`: Boolean flag indicating whether to fetch only group-specific permissions.
    /// - `ressource_list`: Optional reference to a vector of strings representing resource keys for further filtering.
    ///
    /// # Returns
    /// `Result<PermissionInfoListWithCount, Error>`: On success, returns a `PermissionInfoListWithCount` struct,
    /// which includes a vector of `PermissionInfo` and the total count of permissions. On failure, returns an `Error`.
    ///
    /// # Errors
    /// This function might return an error if there are issues with database connectivity, transaction execution,
    /// or query operations.
    ///
    /// # Example
    /// ```
    /// let pagination = PagingSchema { page: 0, limit: 10 };
    /// let result = instance.fetch_user_permissions(&user_id, &pagination, &Some(group_id), false, &None);
    /// match result {
    ///     Ok(permission_info_list) => println!("Fetched permissions: {:?}", permission_info_list),
    ///     Err(e) => println!("Error fetching user permissions: {}", e),
    /// }
    /// ```
    pub fn fetch_user_permissions(
        &self,
        user_id: &Uuid,
        pagination: &PagingSchema,
        group_id: &Option<Uuid>,
        only_group: bool,
        ressource_list: &Option<Vec<String>>,
    ) -> Result<PermissionInfoListWithCount, Error> {
        use crate::schema::user_permissions;
        use crate::schema::ressources;
        use crate::schema::user_access_types;

        let conn 
        = &mut self.pool.get().unwrap();

        let mut filter_query: Box<dyn BoxableExpression<_, diesel::pg::Pg, SqlType = Bool>> = Box::new( 
            user_permissions::user_id.eq(user_id)
        );

        let check_global_permission = user_permissions::group_id.is_null();

        if group_id.is_some() {
            let group_id_value = group_id.unwrap_or_default();

            let mut filter_sub_query: Box<dyn BoxableExpression<_, diesel::pg::Pg, SqlType = Bool>>
                 = Box::new(
                    user_permissions::group_id.eq(group_id_value).is_not_null()
                );

            if !only_group {
                filter_sub_query = Box::new(
                    filter_sub_query.or(
                        check_global_permission
                    )
                )
            }

            filter_query = Box::new(
                filter_query.and(
                    filter_sub_query
                )
            );
        } else {
            filter_query = Box::new(filter_query.and(check_global_permission));
        }

        let offset = pagination.page * pagination.limit;

        conn.transaction(|conn| {
            let total_count = match user_permissions::table
                .inner_join(ressources::table)
                .select(count_star())
                .filter(&filter_query)
                .first(conn) 
            {
                Ok(v) => v,
                Err(err) => return Err(err),
            };

            if ressource_list.is_some() {
                filter_query = Box::new(
                    filter_query
                    .and(
                        ressources::key_value.eq_any(ressource_list.as_ref().unwrap())
                    )
                );
            }

            let user_permission_list: Vec<(Uuid, String, String, Option<Uuid>)> = match user_permissions::table
                .inner_join(ressources::table)
                .select((user_permissions::id,
                    ressources::key_value, ressources::key_name, 
                    user_permissions::group_id,
                ))
                .filter(filter_query)
                .limit(pagination.limit.into())
                .offset(offset.into())
                .load(conn) 
            {
                Ok(v) => v,
                Err(err) => return Err(err),
            };

            let mut permission_list: Vec<PermissionInfo> = vec![];

            for user_permission in user_permission_list {
                let mut addon_list: Vec<UserAccessType> = user_access_types::table
                    .select((
                        user_access_types::access_type,
                        user_access_types::permission,
                        user_access_types::set_permission,
                        user_access_types::set_set_permission,
                    ))
                    .filter(
                        user_access_types::user_permission_id.eq(user_permission.0)
                    )
                    
                    .load(conn).unwrap();

                    permission_list.push(
                    PermissionInfo {
                        key_value: user_permission.1, 
                        key_name: user_permission.2, 
                        access_types: addon_list,
                        group_id: user_permission.3,
                    }
                );
            }

            Ok(PermissionInfoListWithCount { 
                permission_list, 
                total_count, 
            })
        })
    }

    pub fn create_role(
        &self,
        new_role: &NewRole,
    ) -> Result<usize, Error> {
        use crate::schema::roles;
        
        let conn = 
            &mut self.pool.get().unwrap();

        diesel::insert_into(roles::table)
            .values(new_role)
            .on_conflict_do_nothing()
            .execute(conn)
    }

    pub fn update_permission_on_role(
        &self,
        update_role: &UpdateRolePermission,
    ) -> Result<usize, Error> {
        use crate::schema::role_permissions;
        use crate::schema::role_access_types;

        let conn = &mut self.pool.get().unwrap();

        conn.transaction(|conn| {
            let role_permission_id: Uuid = diesel::insert_into(role_permissions::table)
            .values(&update_role.role_permission).on_conflict((
                role_permissions::role,
                role_permissions::ressource,
            )).do_nothing()
            .returning(role_permissions::id)
            .get_result(conn).unwrap();
                
            let mut updated = 0;
            for role_access_type in &update_role.role_access_types {
                if role_access_type.permission.is_none() 
                    && role_access_type.set_permission.is_none() 
                    && role_access_type.set_set_permission.is_none() {
                    continue;
                }

                let exists: Option<AccessType> = role_access_types::table
                    .select(role_access_types::access_type)
                    .filter(
                        role_access_types::role_permission_id.eq(role_permission_id)
                        .and(role_access_types::access_type.eq(role_access_type.access_type)))
                    .first(conn).optional().unwrap();

                let new_role_access_type: NewRoleAccessType = NewRoleAccessType {
                    access_type: role_access_type.access_type,
                    permission: role_access_type.permission,
                    set_permission: role_access_type.set_permission,
                    set_set_permission: role_access_type.set_set_permission,
                    role_permission_id,
                };

                //  cant use inser_into.on_conflict because false, false, flase permissions will break it because of the triggers
                // it will crash while trying insert_into
                if exists.is_some() {
                    updated += diesel::update(role_access_types::table)
                    .set(new_role_access_type)
                    .filter(
                        role_access_types::role_permission_id.eq(role_permission_id)
                        .and(role_access_types::access_type.eq(&role_access_type.access_type))
                    )
                    .execute(conn).unwrap()
                } else if 
                    role_access_type.permission.is_some() 
                    || role_access_type.set_permission.is_some() 
                    || role_access_type.set_set_permission.is_some() {

                    let perm = role_access_type.permission.as_ref().unwrap_or(&false);
                    let set_perm = role_access_type.set_permission.as_ref().unwrap_or(&false);
                    let set_set_perm = role_access_type.set_set_permission.as_ref().unwrap_or(&false);

                    if (perm.clone() || set_perm.clone() || set_set_perm.clone()) {
                        updated += diesel::insert_into(role_access_types::table)
                        .values(
                            new_role_access_type
                        ).on_conflict_do_nothing()
                        .execute(conn).unwrap();
                    }
                }
            }

            Ok(updated)
        })
    }
}