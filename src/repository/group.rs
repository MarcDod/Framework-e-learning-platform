use std::collections::{HashSet, HashMap};

use chrono::{NaiveDate, NaiveDateTime};
// Documentation was created by ChatGPT
use diesel::{
    dsl::{self, count_star}, prelude::*, result::Error, sql_types::{Bool, BigInt, Timestamptz}, upsert::excluded,
};

use itertools::Itertools;
use uuid::Uuid;

use crate::{models::{
        groups::{
            CreateGroup, GroupInfo, GroupInfoListWithCount, GroupMetaData, NewGroup,
            NewGroupMember, NewUserPermission,
        },
        members::{MemberInfo, MemberListWithCount},
        util::{PagingSchema, State, AnswerState, TaskPackageType, Visibility, AccessType}, task::{NewTask, Task, NewTempTask}, answer::{NewAnswer, CreatedAnswer, Answer}, task_package::{NewTaskPackage, CreatedTaskPackage, TaskPackage, TaskPackageUserStatisticValue}, solution_attempts::{CreatedSolutionAttempt, NewSolutionAttempt, SolutionAttempt, AnswerEntry}, permissions::{RoleAccesType, NewUserAccessType, OptionalUserAccessType, UpdateUserAccessType}, roles::{NewRole, UpdateRolePermission},
    }, repository::group};

use super::postgres::DBPool;

#[derive(Clone)]
pub struct GroupRepo {
    pool: DBPool,
}

pub const CREATED_GROUP_ROLE_KEY: &str = "created_group";
pub const ADD_MEMBER_ROLE_KEY: &str = "add_member";
pub const READ_GROUP_INFO: &str = "group";

impl GroupRepo {
    pub fn new(pool: DBPool) -> Self {
        GroupRepo { pool }
    }
    
    /// Fetches permissions associated with a specific role.
    ///
    /// This function retrieves a list of permissions for a given role identified by its key.
    /// It performs an inner join between the 'roles' and 'role_permissions' tables to fetch
    /// the relevant permissions. Each permission is represented as a tuple containing the
    /// permission's UUID and the resource it pertains to.
    ///
    /// # Parameters
    /// - `&self`: Reference to the current instance of the struct containing this method.
    /// - `role_key`: Reference to a string slice representing the key of the role whose permissions are to be fetched.
    ///
    /// # Returns
    /// `Result<Vec<(Uuid, String)>, Error>`: On success, returns a vector of tuples, where each tuple contains the UUID and
    /// resource string of a permission. On failure, returns an `Error`.
    ///
    /// # Errors
    /// This function might return an error if there are issues with database connectivity or the execution of the query.
    ///
    /// # Example
    /// ```
    /// let role_key = "admin";
    /// let result = instance.fetch_role_permissions(role_key);
    /// match result {
    ///     Ok(permissions) => println!("Permissions for role '{}': {:?}", role_key, permissions),
    ///     Err(e) => println!("Error fetching permissions for role '{}': {}", role_key, e),
    /// }
    /// ```
    fn fetch_role_permissions(&self, role_key: &str) -> Result<Vec<(Uuid, String)>, Error> {
        use crate::schema::role_permissions;
        use crate::schema::roles;

        let conn = &mut self.pool.get().unwrap();

        roles::table
            .inner_join(role_permissions::table)
            .select((
                role_permissions::id,
                role_permissions::ressource,
            ))
            .filter(roles::value_key.eq(role_key))
            .load(conn)
    }


    /// Sets or updates user permissions and access types.
    ///
    /// This function inserts a new user permission into the 'user_permissions' table based on the
    /// provided `NewUserPermission` struct. If a conflict occurs, it does nothing (ignores the conflict).
    /// The function then fetches the ID of the newly inserted or existing permission and uses it to
    /// insert or update access types in the 'user_access_types' table. Access types are specified in the
    /// `access_types` parameter. The function operates within a database transaction to ensure atomicity.
    ///
    /// # Parameters
    /// - `&self`: Reference to the current instance of the struct containing this method.
    /// - `new_ressource`: Reference to a `NewUserPermission` struct containing the new permission details.
    /// - `access_types`: Reference to a vector of `OptionalUserAccessType` structs representing the access types to be set or updated.
    ///
    /// # Returns
    /// `Result<usize, Error>`: On success, returns the number of rows affected (as `usize`).
    /// On failure, returns an `Error`.
    ///
    /// # Errors
    /// This function might return an error if there are issues with database connectivity,
    /// transaction execution, or the insertion/update operations.
    ///
    /// # Example
    /// ```
    /// let new_permission = NewUserPermission { /* fields */ };
    /// let access_types = vec![/* access types data */];
    /// let result = instance.user_set_permission(&new_permission, &access_types);
    /// match result {
    ///     Ok(rows_updated) => println!("Number of rows updated: {}", rows_updated),
    ///     Err(e) => println!("Error setting user permission: {}", e),
    /// }
    /// ```
    pub fn user_set_permission(
        &self, 
        new_ressource: &NewUserPermission, 
        access_types: &Vec<OptionalUserAccessType>
    ) -> Result<usize, Error> {
        use crate::schema::user_permissions;
        use crate::schema::user_access_types;

        let conn = &mut self.pool.get().unwrap();

        conn.transaction(|conn| {
            diesel::insert_into(user_permissions::table)
            .values((
                user_permissions::user_id.eq(new_ressource.user_id),
                user_permissions::ressource.eq(&new_ressource.ressource),
                user_permissions::group_id.eq(new_ressource.group_id)
            )).on_conflict((
                user_permissions::user_id,
                user_permissions::ressource,
                user_permissions::group_id,
            )).do_nothing()
            .execute(conn)?;
                

            let user_permission_id = user_permissions::table
                .select(user_permissions::id)
                .filter(
                    user_permissions::user_id.eq(new_ressource.user_id)
                    .and(user_permissions::ressource.eq(&new_ressource.ressource))
                    .and(user_permissions::group_id.is_not_distinct_from(new_ressource.group_id))
                ).first(conn).unwrap();

            let mut updated = 0;
            for user_access_type in access_types {
                if user_access_type.permission.is_none() 
                    && user_access_type.set_permission.is_none() 
                    && user_access_type.set_set_permission.is_none() {
                    continue;
                }

                let new_access_type = NewUserAccessType {
                    access_type: user_access_type.access_type,
                    permission: user_access_type.permission,
                    set_permission: user_access_type.set_permission,
                    set_set_permission: user_access_type.set_set_permission,
                    user_permission_id,
                };

                let exists: Option<AccessType> = user_access_types::table
                    .select(user_access_types::access_type)
                    .filter(
                        user_access_types::user_permission_id.eq(user_permission_id)
                        .and(user_access_types::access_type.eq(user_access_type.access_type)))
                    .first(conn).optional().unwrap();

                //  cant use inser_into.on_conflict because false, false, flase permissions will break it because of the triggers
                // it will crash while trying insert_into
                if exists.is_some() {
                    updated += diesel::update(user_access_types::table)
                    .set(UpdateUserAccessType {
                        permission: user_access_type.permission,
                        set_permission: user_access_type.set_permission,
                        set_set_permission: user_access_type.set_set_permission 
                    })
                    .filter(
                        user_access_types::user_permission_id.eq(user_permission_id)
                        .and(user_access_types::access_type.eq(user_access_type.access_type))
                    )
                    .execute(conn).unwrap()
                } else if 
                    user_access_type.permission.is_some() 
                    || user_access_type.set_permission.is_some() 
                    || user_access_type.set_set_permission.is_some() {

                    let perm = user_access_type.permission.as_ref().unwrap_or(&false);
                    let set_perm = user_access_type.set_permission.as_ref().unwrap_or(&false);
                    let set_set_perm = user_access_type.set_set_permission.as_ref().unwrap_or(&false);

                    if (perm.clone() || set_perm.clone() || set_set_perm.clone()) {
                        updated += diesel::insert_into(user_access_types::table)
                        .values(
                            &new_access_type
                        ).on_conflict_do_nothing()
                        .execute(conn).unwrap();
                    }
                }
            }

            Ok(updated)
        })
    }

    /// Creates a new group and assigns permissions to the creator.
    ///
    /// This function creates a new group based on the provided 
    /// information and assigns specific permissions
    /// to the user who is creating the group. The function performs 
    /// a transaction that includes creating the
    /// group, retrieving information about the created group, 
    /// and assigning the creator relevant permissions.
    ///
    /// # Arguments
    ///
    /// * `new_group` - A reference to a `NewGroup` struct containing 
    ///                 information about the new group,
    ///                 including the group name, creator's ID, and 
    ///                 any additional metadata.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `CreateGroup` struct, which includes 
    /// the following information about the created group:
    ///
    /// 1. `Uuid` - The unique identifier of the group.
    /// 2. `String` - The name of the group.
    /// 3. `NaiveDateTime` - The timestamp indicating when the group 
    /// was created.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with 
    /// the database connection or if the specified
    /// creator does not have the required permissions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use diesel::prelude::*;
    ///
    /// let db_conn = PgRepo::establish_connection("database_url");
    /// let group_repo = db_conn.new_group_repo();
    /// let new_group = NewGroup {
    ///     name: "New Group Name".to_string(),
    ///     created_from: Uuid::new_v4(),
    ///     // Additional metadata...
    /// };
    ///
    /// match group_repo.create_group(&new_group) {
    ///     Ok(created_group) => {
    ///         println!("Group ID: {}, Group Name: {}, Created At: {}",
    ///             created_group.id, created_group.name, created_group.created_at);
    ///     },
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    ///
    /// In this example, the function is used to create a new group, assign permissions to the creator, and print information
    /// about the created group.
    pub fn create_group(&self, new_group: &NewGroup) -> Result<CreateGroup, Error> {
        use crate::schema::groups;
        use crate::schema::user_permissions;
        use crate::schema::user_access_types;
        use crate::schema::role_access_types;

        let conn = &mut self.pool.get().unwrap();

        let role_permissions = match self.fetch_role_permissions(CREATED_GROUP_ROLE_KEY) {
            Ok(v) => v,
            Err(err) => return Err(err),
        };

        conn.transaction(|conn| {
            diesel::insert_into(groups::table)
                .values(new_group)
                .execute(conn)?;

            let created_group: CreateGroup = match groups::table
                .select((groups::id, groups::name, groups::created_at, groups::parent))
                .filter(groups::id.eq(&new_group.id))
                .first(conn)
            {
                Ok(v) => v,
                Err(err) => return Err(err),
            };

            for role_permission in role_permissions {

                let access_types: Vec<RoleAccesType> = role_access_types::table
                    .select((
                        role_access_types::access_type,
                        role_access_types::set_permission,
                        role_access_types::set_set_permission,
                        role_access_types::permission,
                    )).filter(
                        role_access_types::role_permission_id.eq(role_permission.0)
                    ).load(conn).unwrap();

                let user_permission_id: Uuid = diesel::insert_into(user_permissions::table)
                    .values((
                        user_permissions::user_id.eq(&new_group.created_from),
                        user_permissions::ressource.eq(&role_permission.1),
                        user_permissions::group_id.eq(&created_group.id),
                    ))
                    .on_conflict_do_nothing()
                    .returning(user_permissions::id)
                    .get_result(conn).unwrap();

                diesel::insert_into(user_access_types::table)
                    .values(access_types.into_iter().map(|user_access_type| NewUserAccessType {
                        access_type: user_access_type.access_type,
                        permission: Some(user_access_type.permission),
                        set_permission: Some(user_access_type.set_permission),
                        set_set_permission: Some(user_access_type.set_set_permission),
                        user_permission_id,
                    }).collect::<Vec<NewUserAccessType>>()
                ).on_conflict((
                    user_access_types::access_type,
                    user_access_types::user_permission_id,
                )).do_update()
                .set((
                    user_access_types::access_type.eq(excluded(user_access_types::access_type)),
                    user_access_types::user_permission_id.eq(excluded(user_access_types::user_permission_id))
                )).execute(conn)?;
            };

            Ok(created_group)
        })
    }

    /// Fetches basic information for an active group based on its 
    /// unique identifier.
    ///
    /// This function retrieves basic information for a specified 
    /// active group, including its unique
    /// identifier and name. The group is identified by its unique ID, 
    /// and only active groups are
    /// considered in the query.
    ///
    /// # Arguments
    ///
    /// * `group_id` - The unique identifier of the active group for 
    /// which information is to be fetched.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `GroupInfo` struct, which includes the 
    /// following basic information:
    ///
    /// 1. `Uuid` - The unique identifier of the group.
    /// 2. `String` - The name of the group.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with the 
    /// database connection or if the specified
    /// active group does not exist.
    ///
    /// # Example
    ///
    /// ```rust
    /// use diesel::prelude::*;
    ///
    /// let db_conn = PgRepo::establish_connection("database_url");
    /// let group_repo = db_conn.new_group_repo();
    /// let group_id = Uuid::new_v4(); // Replace with the actual group ID.
    ///
    /// match group_repo.fetch_active_group_info(group_id) {
    ///     Ok(group_info) => {
    ///         println!("Group ID: {}, Group Name: {}", group_info.id, group_info.name);
    ///     },
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    ///
    /// In this example, the function is used to fetch basic information for an active group and print the retrieved information.
    pub fn fetch_active_group_info(&self, group_id: Uuid) -> Result<GroupInfo, Error> {
        use crate::schema::groups;

        let conn = &mut self.pool.get().unwrap();

        groups::table
            .select((groups::id, groups::name, groups::parent))
            .filter(
                groups::id
                    .eq(&group_id)
                    .and(groups::state.eq(State::Active)),
            )
            .first(conn)
    }

    /// Fetches metadata for an active group based on its unique 
    /// identifier.
    ///
    /// This function retrieves metadata for a specified active group, 
    /// including information such as
    /// its creation and update timestamps. The group is identified by 
    /// its unique ID, and only active
    /// groups are considered in the query.
    ///
    /// # Arguments
    ///
    /// * `group_id` - The unique identifier of the active group for 
    /// which metadata is to be fetched.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `GroupMetaData` struct, which includes 
    /// the following metadata information:
    ///
    /// 1. `Uuid` - The unique identifier of the group.
    /// 2. `Uuid` - The unique identifier representing the entity that 
    /// created the group.
    /// 3. `Uuid` - The unique identifier representing the entity that 
    /// last updated the group.
    /// 4. `NaiveDateTime` - The timestamp indicating when the group 
    /// was created.
    /// 5. `NaiveDateTime` - The timestamp indicating when the group 
    /// was last updated.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with 
    /// the database connection or if the specified
    /// active group does not exist.
    ///
    /// # Example
    ///
    /// ```rust
    /// use diesel::prelude::*;
    ///
    /// let db_conn = PgRepo::establish_connection("database_url");
    /// let group_repo = db_conn.new_group_repo();
    /// let group_id = Uuid::new_v4(); // Replace with the actual group ID.
    ///
    /// match group_repo.fetch_active_group_meta_data(group_id) {
    ///     Ok(group_meta_data) => {
    ///         println!("Group ID: {}, Created From: {}, Updated From: {}, Created At: {}, Updated At: {}",
    ///             group_meta_data.id, group_meta_data.created_from, group_meta_data.updated_from,
    ///             group_meta_data.created_at, group_meta_data.updated_at);
    ///     },
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    ///
    /// In this example, the function is used to fetch metadata for an active group and print the retrieved information.
    pub fn fetch_active_group_meta_data(&self, group_id: Uuid) -> Result<GroupMetaData, Error> {
        use crate::schema::groups;

        let conn = &mut self.pool.get().unwrap();

        groups::table
            .select((
                groups::id,
                groups::created_from,
                groups::updated_from,
                groups::created_at,
                groups::updated_at,
            ))
            .filter(
                groups::id
                    .eq(&group_id)
                    .and(groups::state.eq(State::Active)),
            )
            .first(conn)
    }

    /// Removes a user from a group, revoking their associated 
    /// permissions.
    ///
    /// This function removes a user from a group, including revoking 
    /// any permissions associated with the user
    /// for that group. The function performs a transaction that 
    /// includes fetching the member information,
    /// deleting the user from the group members, and revoking the 
    /// user's permissions for the group.
    ///
    /// # Arguments
    ///
    /// * `remove_member_id` - The unique identifier of the group 
    /// member to be removed.
    /// * `group_id` - The unique identifier of the group from 
    /// which the user is to be removed.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `MemberInfo` struct, which includes:
    ///
    /// 1. `Uuid` - The unique identifier of the removed group member.
    /// 2. `Uuid` - The unique identifier of the user.
    /// 3. `String` - The name of the user.
    /// 4. `String` - The email address of the user.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with 
    /// the database connection or if the specified
    /// member or group does not exist.
    ///
    /// # Example
    ///
    /// ```rust
    /// use diesel::prelude::*;
    ///
    /// let db_conn = PgRepo::establish_connection("database_url");
    /// let group_repo = db_conn.new_group_repo();
    /// let remove_member_id = Uuid::new_v4(); // Replace with the actual member ID to be removed.
    /// let group_id = Uuid::new_v4(); // Replace with the actual group ID.
    ///
    /// match group_repo.remove_user_from_group(&remove_member_id, &group_id) {
    ///     Ok(removed_member) => {
    ///         println!("Removed Member ID: {}, User ID: {}, Name: {}, Email: {}",
    ///             removed_member.id, removed_member.user_id, removed_member.name, removed_member.email);
    ///     },
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    ///
    /// In this example, the function is used to remove a user from 
    /// a group and print the removed member information.
    pub fn remove_user_from_group(
        &self,
        remove_member_id: &Uuid,
        group_id: &Uuid,
    ) -> Result<MemberInfo, Error> {
        use crate::schema::group_members;
        use crate::schema::users;
        use crate::schema::user_permissions;

        let conn = &mut self.pool.get().unwrap();

        conn.transaction(|conn| {
            let member: MemberInfo = group_members::table
                .inner_join(users::table)
                .select((group_members::id, users::id, users::name, users::email))
                .filter(
                    group_members::id
                        .eq(remove_member_id)
                        .and(group_members::group_id.eq(group_id)),
                )
                .first(conn)
                .unwrap();

            diesel::delete(group_members::table)
                .filter(
                    group_members::id
                        .eq(remove_member_id)
                        .and(group_members::group_id.eq(group_id)),
                )
                .execute(conn)?;

            diesel::delete(user_permissions::table)
                .filter(
                    user_permissions::group_id
                        .eq(group_id)
                        .and(user_permissions::user_id.eq(&member.user_id)),
                )
                .execute(conn)?;

            Ok(member)
        })
    }

    /// Adds a new user to a group, granting them specified permissions.
    ///
    /// This function adds a new user to a group, along with 
    /// granting the user specified permissions for the group.
    /// The function performs a transaction that includes adding 
    /// the user to the group members, retrieving the created
    /// member information, and assigning the user relevant 
    /// permissions for the group.
    ///
    /// # Arguments
    ///
    /// * `new_member` - A reference to a `NewGroupMember` struct 
    /// containing information about the new member,
    ///                  including the user ID, group ID, and any 
    /// additional metadata.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `MemberInfo` struct, which includes:
    ///
    /// 1. `Uuid` - The unique identifier of the group member.
    /// 2. `Uuid` - The unique identifier of the user.
    /// 3. `String` - The name of the user.
    /// 4. `String` - The email address of the user.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with 
    /// the database connection or if the user is
    /// already a member of the group.
    ///
    /// # Example
    ///
    /// ```rust
    /// use diesel::prelude::*;
    ///
    /// let db_conn = PgRepo::establish_connection("database_url");
    /// let group_repo = db_conn.new_group_repo();
    /// let new_member = NewGroupMember {
    ///     user_id: Uuid::new_v4(),
    ///     group_id: Uuid::new_v4(),
    ///     // Additional metadata...
    /// };
    ///
    /// match group_repo.add_user_to_group(&new_member) {
    ///     Ok(created_member) => {
    ///         println!("Member ID: {}, User ID: {}, Name: {}, Email: {}",
    ///             created_member.id, created_member.user_id, created_member.name, created_member.email);
    ///     },
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    ///
    /// In this example, the function is used to add a new user to a group and print the created member information.
    pub fn add_user_to_group(&self, new_member: &NewGroupMember) -> Result<MemberInfo, Error> {
        use crate::schema::group_members;
        use crate::schema::users;
        use crate::schema::user_permissions;
        use crate::schema::role_access_types;
        use crate::schema::user_access_types;

        let conn = &mut self.pool.get().unwrap();

        let role_permissions = match self.fetch_role_permissions(ADD_MEMBER_ROLE_KEY) {
            Ok(v) => v,
            Err(err) => return Err(err),
        };

        conn.transaction(|conn| {
            diesel::insert_into(group_members::table)
                .values(new_member)
                .execute(conn)?;

            let created_member: MemberInfo = match group_members::table
                .inner_join(users::table.on(users::id.eq(group_members::user_id)))
                .select((group_members::id, users::id, users::name, users::email))
                .filter(
                    group_members::user_id
                        .eq(new_member.user_id)
                        .and(group_members::group_id.eq(new_member.group_id)),
                )
                .first(conn)
            {
                Ok(v) => v,
                Err(err) => return Err(err),
            };

            for role_permission in role_permissions {

                let access_types: Vec<RoleAccesType> = role_access_types::table
                    .select((
                        role_access_types::access_type,
                        role_access_types::set_permission,
                        role_access_types::set_set_permission,
                        role_access_types::permission,
                    )).filter(
                        role_access_types::role_permission_id.eq(role_permission.0)
                    ).load(conn).unwrap();

                let user_permission_id: Uuid = diesel::insert_into(user_permissions::table)
                    .values((
                        user_permissions::user_id.eq(&new_member.user_id),
                        user_permissions::ressource.eq(&role_permission.1),
                        user_permissions::group_id.eq(&new_member.group_id),
                    ))
                    .on_conflict_do_nothing()
                    .returning(user_permissions::id)
                    .get_result(conn).unwrap();

                diesel::insert_into(user_access_types::table)
                    .values(access_types.into_iter().map(|user_access_type| NewUserAccessType {
                        access_type: user_access_type.access_type,
                        permission: Some(user_access_type.permission),
                        set_permission: Some(user_access_type.set_permission),
                        set_set_permission: Some(user_access_type.set_set_permission),
                        user_permission_id,
                    }).collect::<Vec<NewUserAccessType>>()
                ).on_conflict((
                    user_access_types::access_type,
                    user_access_types::user_permission_id,
                )).do_update()
                .set((
                    user_access_types::access_type.eq(excluded(user_access_types::access_type)),
                    user_access_types::user_permission_id.eq(excluded(user_access_types::user_permission_id))
                )).execute(conn)?;
            };

            Ok(created_member)
        })
    }

    /// Fetches a paginated list of groups where a user has 
    /// specific rights, optionally filtered by group IDs.
    ///
    /// This function retrieves a paginated list of groups for 
    /// which a specified user has specific rights,
    /// providing both a subset of group information and the total 
    /// count of eligible groups. Additionally,
    /// you can provide a list of specific group IDs to further 
    /// filter the results.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The unique identifier of the user for whom 
    /// groups with specific rights are to be fetched.
    /// * `pagination` - A reference to a `PagingSchema` struct that 
    /// defines the pagination parameters,
    ///                 such as page number and limit.
    /// * `group_ids` - An optional vector of group IDs to filter the 
    /// fetched groups. If `None`, all eligible
    ///                groups will be retrieved.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `GroupInfoListWithCount` struct, 
    /// which includes:
    ///
    /// 1. `Vec<(Uuid, String)>` - A paginated list of tuples, where 
    /// each tuple represents a group. The tuple
    ///    includes the group's unique ID and name.
    /// 2. `usize` - The total count of groups for which the user has 
    /// specific rights.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with the 
    /// database connection.
    ///
    /// # Example
    ///
    /// ```rust
    /// use diesel::prelude::*;
    ///
    /// let db_conn = PgRepo::establish_connection("database_url");
    /// let group_repo = db_conn.new_group_repo();
    /// let user_id = Uuid::new_v4();
    /// let pagination = PagingSchema {
    ///     page: 1,
    ///     limit: 10,
    /// };
    /// let group_ids = Some(vec![Uuid::new_v4(), Uuid::new_v4()]);
    ///
    /// match group_repo.fetch_groups_where_user_has_rights(&user_id, &pagination, &group_ids) {
    ///     Ok(group_info_list_with_count) => {
    ///         let GroupInfoListWithCount { group_info_list, total_count } = group_info_list_with_count;
    ///         for (group_id, group_name) in group_info_list {
    ///             println!("Group ID: {}, Group Name: {}", group_id, group_name);
    ///         }
    ///         println!("Total Groups with Rights: {}", total_count);
    ///     },
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    ///
    /// In this example, the function is used to fetch a paginated list of groups for which a user has specific
    /// rights, optionally filtered by specific group IDs, and the retrieved information is printed.
    pub fn fetch_groups_where_user_has_rights(
        &self,
        user_id: &Uuid,
        pagination: &PagingSchema,
        group_ids: &Option<Vec<Uuid>>,
    ) -> Result<GroupInfoListWithCount, Error> {
        use crate::schema::groups;
        use crate::schema::ressources;
        use crate::schema::user_permissions;
        use crate::schema::user_access_types;

        let conn = &mut self.pool.get().unwrap();

        let offset = pagination.page * pagination.limit;

        let mut filter_query: Box<dyn BoxableExpression<_, diesel::pg::Pg, SqlType = Bool>> =
            Box::new(
                user_permissions::user_id
                    .eq(user_id)
                    .and(ressources::key_value.eq(READ_GROUP_INFO))
                    .and(user_access_types::access_type.eq(AccessType::Read))
                    .and(user_access_types::permission.eq(true))
                    .and(groups::state.eq(State::Active)),
            );

        conn.transaction(|conn| {
            let total_count = match groups::table
                .inner_join(
                    user_permissions::table
                    .inner_join(ressources::table)
                    .inner_join(user_access_types::table)
                )
                .select(count_star())
                .filter(&filter_query)
                .first(conn)
            {
                Ok(v) => v,
                Err(err) => return Err(err),
            };

            if group_ids.is_some() {
                filter_query =
                    Box::new(filter_query.and(groups::id.eq_any(group_ids.as_ref().unwrap())));
            }

            let group_info_list = match groups::table
                .inner_join(
                    user_permissions::table
                    .inner_join(ressources::table)
                    .inner_join(user_access_types::table)
                )
                .select((groups::id, groups::name, groups::parent))
                .limit(pagination.limit.into())
                .offset(offset.into())
                .filter(filter_query)
                .load(conn)
            {
                Ok(v) => v,
                Err(err) => return Err(err),
            };

            Ok(GroupInfoListWithCount {
                group_info_list,
                total_count,
            })
        })
    }

    /// Fetches a paginated list of active groups along with the 
    /// total count, optionally filtered by group IDs.
    ///
    /// This function retrieves a paginated list of active groups, 
    /// providing both a subset of group information
    /// and the total count of active groups. Additionally, you can 
    /// provide a list of specific group IDs to
    /// further filter the results.
    ///
    /// # Arguments
    ///
    /// * `pagination` - A reference to a `PagingSchema` struct that 
    /// defines the pagination parameters,
    ///                 such as page number and limit.
    /// * `group_ids` - An optional vector of group IDs to filter the 
    /// fetched groups. If `None`, all active
    ///                groups will be retrieved.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `GroupInfoListWithCount` struct, which 
    /// includes:
    ///
    /// 1. `Vec<(Uuid, String)>` - A paginated list of tuples, where each 
    /// tuple represents a group. The tuple
    ///    includes the group's unique ID and name.
    /// 2. `usize` - The total count of active groups.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with 
    /// the database connection.
    ///
    /// # Example
    ///
    /// ```rust
    /// use diesel::prelude::*;
    ///
    /// let db_conn = PgRepo::establish_connection("database_url");
    /// let group_repo = db_conn.new_group_repo();
    /// let pagination = PagingSchema {
    ///     page: 1,
    ///     limit: 10,
    /// };
    /// let group_ids = Some(vec![Uuid::new_v4(), Uuid::new_v4()]);
    ///
    /// match group_repo.fetch_all_active_groups(&pagination, &group_ids) {
    ///     Ok(group_info_list_with_count) => {
    ///         let GroupInfoListWithCount { group_info_list, total_count } = group_info_list_with_count;
    ///         for (group_id, group_name) in group_info_list {
    ///             println!("Group ID: {}, Group Name: {}", group_id, group_name);
    ///         }
    ///         println!("Total Active Groups: {}", total_count);
    ///     },
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    ///
    /// In this example, the function is used to fetch a paginated list of active groups, optionally filtered
    /// by specific group IDs, and the retrieved information is printed.
    pub fn fetch_all_active_groups(
        &self,
        pagination: &PagingSchema,
        group_ids: &Option<Vec<Uuid>>,
    ) -> Result<GroupInfoListWithCount, Error> {
        use crate::schema::groups;

        let conn = &mut self.pool.get().unwrap();

        let offset = pagination.page * pagination.limit;

        let mut filter_query: Box<dyn BoxableExpression<_, diesel::pg::Pg, SqlType = Bool>> =
            Box::new(groups::state.eq(State::Active));

        if group_ids.is_some() {
            filter_query = Box::new(groups::id.eq_any(group_ids.as_ref().unwrap()));
        }

        conn.transaction(|conn| {
            let group_info_list = match groups::table
                .select((groups::id, groups::name, groups::parent))
                .limit(pagination.limit.into())
                .offset(offset.into())
                .filter(filter_query)
                .load(conn)
            {
                Ok(v) => v,
                Err(err) => return Err(err),
            };

            let total_count = match groups::table.select(count_star()).first(conn) {
                Ok(v) => v,
                Err(err) => return Err(err),
            };

            Ok(GroupInfoListWithCount {
                group_info_list,
                total_count,
            })
        })
    }

    /// Fetches a paginated list of members belonging to a 
    /// specific group along with the total member count.
    ///
    /// This function retrieves a list of members who belong to a 
    /// specified group, providing both
    /// a paginated subset of members and the total count of members in 
    /// the group.
    ///
    /// # Arguments
    ///
    /// * `group_id` - The unique identifier of the group for which 
    /// members are to be fetched.
    /// * `pagination` - A reference to a `PagingSchema` struct that 
    /// defines the pagination parameters,
    ///                 such as page number and limit.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `MemberListWithCount` struct, which 
    /// includes:
    ///
    /// 1. `Vec<(Uuid, Uuid, String, String)>` - A paginated list of 
    /// tuples, where each tuple represents
    ///    a member in the group. The tuple includes the member's 
    /// unique ID, user ID, name, and email.
    /// 2. `usize` - The total count of members in the specified group.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with 
    /// the database connection or if
    /// the specified group does not exist.
    ///
    /// # Example
    ///
    /// ```rust
    /// use diesel::prelude::*;
    ///
    /// let db_conn = PgRepo::establish_connection("database_url");
    /// let group_repo = db_conn.new_group_repo();
    /// let group_id = Uuid::new_v4();
    /// let pagination = PagingSchema {
    ///     page: 1,
    ///     limit: 10,
    /// };
    ///
    /// match group_repo.fetch_all_member_of_group(&group_id, &pagination) {
    ///     Ok(member_list_with_count) => {
    ///         let MemberListWithCount { member_list, total_count } = member_list_with_count;
    ///         for (member_id, user_id, name, email) in member_list {
    ///             println!("Member ID: {}, User ID: {}, Name: {}, Email: {}", member_id, user_id, name, email);
    ///         }
    ///         println!("Total Members in Group: {}", total_count);
    ///     },
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    ///
    /// In this example, the function is used to fetch a paginated list of members belonging to a group,
    /// along with the total count of members in the group, and the retrieved information is printed.
    pub fn fetch_all_member_of_group(
        &self,
        group_id: &Uuid,
        pagination: &PagingSchema,
    ) -> Result<MemberListWithCount, Error> {
        use crate::schema::group_members;
        use crate::schema::users;

        let conn = &mut self.pool.get().unwrap();

        let offset = pagination.page * pagination.limit;

        let filter_query = group_members::group_id.eq(group_id);

        conn.transaction(|conn| {
            let member_list = match group_members::table
                .inner_join(users::table.on(group_members::user_id.eq(users::id)))
                .select((group_members::id, users::id, users::name, users::email))
                .filter(filter_query)
                .offset(offset.into())
                .limit(pagination.limit.into())
                .load(conn)
            {
                Ok(v) => v,
                Err(err) => return Err(err),
            };

            let total_count = match group_members::table
                .select(count_star())
                .filter(filter_query)
                .first(conn)
            {
                Ok(v) => v,
                Err(err) => return Err(err),
            };

            Ok(MemberListWithCount {
                member_list,
                total_count,
            })
        })
    }

    /// Fetches information about all members of a group
    /// with specified member IDs.
    ///
    /// This function retrieves details about members
    /// of a  group, specified by their unique
    /// member IDs, including their user IDs, names, and email addresses.
    ///
    /// # Arguments
    ///
    /// * `group_id` - The unique identifier of the  group.
    /// * `member_ids` - A vector of unique member IDs to retrieve information for.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<MemberInfo>` where each `MemberInfo`
    /// includes the following information:
    ///
    /// * `member_id` - The unique identifier of the group member.
    /// * `user_id` - The unique identifier of the user.
    /// * `name` - The name of the user.
    /// * `email` - The email address of the user.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with the database connection or
    /// if the specified  group or member IDs do not exist.
    ///
    /// # Example
    ///
    /// ```rust
    /// use use repository::postgres::PgRepo; // Import your database connection type.
    ///
    /// let db_conn = PgRepo::establish_connection("database_url");
    /// let group_repo = db_conn.new_group_repo();
    /// let group_id = Uuid::new_v4();
    /// let member_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
    ///
    /// match group_repo.fetch_all_member_of_group_in(&group_id, &member_ids) {
    ///     Ok(member_info) => {
    ///         for member in member_info {
    ///             println!("Member ID: {}, User ID: {}, Name: {}, Email: {}", member.member_id, member.user_id, member.name, member.email);
    ///         }
    ///     },
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    ///
    /// In this example, the function is used to fetch information about members of a  group
    /// based on their unique member IDs, and the retrieved information is printed.
    pub fn fetch_all_member_of_group_in(
        &self,
        group_id: &Uuid,
        member_ids: &Vec<Uuid>,
    ) -> Result<MemberListWithCount, Error> {
        use crate::schema::group_members;
        use crate::schema::users;

        let conn = &mut self.pool.get().unwrap();

        conn.transaction(|conn| {
            let member_list = match group_members::table
                .inner_join(users::table.on(group_members::user_id.eq(users::id)))
                .select((group_members::id, users::id, users::name, users::email))
                .filter(
                    group_members::group_id
                        .eq(group_id)
                        .and(group_members::id.eq_any(member_ids)),
                )
                .load(conn)
            {
                Ok(v) => v,
                Err(err) => return Err(err),
            };

            let total_count = match group_members::table
                .select(count_star())
                .filter(group_members::group_id.eq(group_id))
                .first(conn)
            {
                Ok(v) => v,
                Err(err) => return Err(err),
            };

            Ok(MemberListWithCount {
                member_list,
                total_count,
            })
        })
    }

    /// Marks a group as deleted in the database and logs the user who initiated the action.
    ///
    /// This function updates the status of a group in the database by marking it as deleted and
    /// records the user who initiated the deletion action.
    ///
    /// # Arguments
    ///
    /// * `group_id` - The unique identifier of the group to be marked as deleted.
    /// * `user_id` - The unique identifier of the user who initiated the deletion.
    ///
    /// # Returns
    ///
    /// A `Result` containing the number of affected rows, typically 1 if the group was successfully
    /// marked as deleted, or an error if the operation fails.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with the database connection or if the
    /// specified group does not exist.
    ///
    /// # Example
    ///
    /// ```rust
    /// use use repository::postgres::PgRepo; // Import your database connection type.
    ///
    /// let db_conn = PgRepo::establish_connection("database_url");
    /// let group_repo = db_conn.new_group_repo();
    /// let group_id = Uuid::new_v4();
    /// let user_id = Uuid::new_v4();
    ///
    /// match group_repo.delete_group(&group_id, &user_id) {
    ///     Ok(affected_rows) => {
    ///         if affected_rows == 1 {
    ///             println!("Group marked as deleted.");
    ///         } else {
    ///             println!("Group not found or already marked as deleted.");
    ///         }
    ///     },
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    ///
    /// In this example, the function is used to mark a group as deleted in the database.
    pub fn delete_group(
        &self,
        group_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<usize, Error> {
        use crate::schema::groups;

        let conn = 
            &mut self.pool.get().unwrap();

        diesel::update(
            groups::table
        ).set(
            (groups::state.eq(State::Deleted),
            groups::updated_from.eq(user_id),
            groups::updated_at.eq(dsl::now)),
        ).filter(
            groups::id.eq(group_id)
        ).execute(conn)
    }

    /// Fetches the task type associated with the provided task ID.
    ///
    /// # Arguments
    ///
    /// * `task_id` - A reference to the UUID representing the task ID.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the fetched task type as a `String` or an `Error` if the operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate_name::YourStructName;
    /// use uuid::Uuid;
    ///
    /// let your_struct_instance = YourStructName::new(); // Replace with your actual struct name and instance.
    /// let task_id = Uuid::new_v4();
    ///
    /// match your_struct_instance.fetch_task_type(&task_id) {
    ///     Ok(task_type) => {
    ///         println!("Fetched task type: {}", task_type);
    ///         // Handle the fetched task type
    ///     }
    ///     Err(err) => {
    ///         eprintln!("Error fetching task type: {}", err);
    ///         // Handle the error
    ///     }
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if there is an issue retrieving the database connection or if the query execution fails.
    pub fn fetch_task_type(
        &self,
        task_id: &Uuid,
    ) -> Result<String, Error> {
        use crate::schema::tasks;

        let conn = 
        &mut self.pool.get().unwrap();

        tasks::table
            .select(
                tasks::task_type
            )
            .filter(
                tasks::id.eq(task_id)
            ).first(conn)
    }

    pub fn fetch_task_packages(
        &self,
        group_id: &Uuid,
    ) -> Result<Vec<TaskPackage>, Error> {
        use crate::schema::task_packages;

        let conn = 
            &mut self.pool.get().unwrap();
        
        task_packages::table
            .select((
                task_packages::id,
                task_packages::name,
                task_packages::task_package_type,
            ))
            .filter(
                task_packages::group_id.eq(group_id)
            )
            .load(conn)
    }

    /// Fetches tasks from a specified task collection, optionally filtered by task types.
    ///
    /// This function retrieves tasks associated with a given task collection ID. Optionally, it
    /// can filter the tasks based on a list of task types. If no task types are provided, all
    /// tasks within the collection are fetched. The function queries tasks that are in the 'Active'
    /// state when filtering by task types.
    ///
    /// # Parameters
    /// - `&self`: Reference to the current instance of the struct containing this method.
    /// - `task_package_id`: Reference to a UUID representing the ID of the task collection from which tasks are to be fetched.
    /// - `task_types`: Optional reference to a vector of strings representing the types of tasks to filter by.
    ///
    /// # Returns
    /// `Result<Vec<CollectionTask>, Error>`: On success, returns a vector of `CollectionTask` structs representing the
    /// fetched tasks. On failure, returns an `Error`.
    ///
    /// # Errors
    /// This function might return an error if there are issues with database connectivity or the execution of the query.
    ///
    /// # Example
    /// ```
    /// let optional_task_types = Some(vec![String::from("Type1"), String::from("Type2")]);
    /// let result = instance.fetch_tasks_from_collection(&task_package_id, &optional_task_types);
    /// match result {
    ///     Ok(tasks) => println!("Fetched tasks: {:?}", tasks),
    ///     Err(e) => println!("Error fetching tasks: {}", e),
    /// }
    /// ```
    pub fn fetch_tasks_from_package(
        &self,
        task_package_id: &Uuid,
        group_id: &Uuid,
        task_types: &Option<Vec<String>>,
    ) -> Result<Vec<Task>, Error> {
        use crate::schema::tasks;
        use crate::schema::task_packages;

        let conn = 
            &mut self.pool.get().unwrap();

        let mut filter_query: 
            Box<dyn BoxableExpression<_, diesel::pg::Pg, SqlType = Bool>> =
            Box::new(
                tasks::task_package_id.eq(task_package_id)
                .and(task_packages::group_id.eq(group_id))
            );

        if task_types.is_some() {
            filter_query = Box::new(
                filter_query.and(
                    tasks::task_type.eq_any(task_types.as_ref().unwrap())
                    .and(tasks::state.eq(State::Active))
                )
            );
        }

        tasks::table
            .inner_join(task_packages::table)
            .select(
                (tasks::id, tasks::task_package_id, tasks::task_doc_id, tasks::task_type)
            )
            .filter(filter_query)
            .load(conn)
    }

    pub fn add_tasks_to_package(
        &self,
        task_package_id: &Uuid,
        group_id: &Uuid,
        tasks: &Vec<NewTempTask>,
    ) -> Result<Vec<Task>, Error> {
        use crate::schema::tasks;
        use crate::schema::task_packages;

        let conn = 
            &mut self.pool.get().unwrap();

        conn.transaction(|conn| {
            let task_package: Option<Uuid> = task_packages::table
                .select(task_packages::id)
                .filter(
                    task_packages::id.eq(task_package_id)
                    .and(task_packages::group_id.eq(group_id))
                ).first(conn).optional().unwrap();
            
            if task_package.is_none() {
                return Ok(vec![]);
            }

            diesel::insert_into(tasks::table)
            .values(tasks.into_iter().map(|task| NewTask {
                task_package_id: task_package_id,
                task_doc_id: &task.task_doc_id,
                task_type: &task.task_type,
            }).collect::<Vec<NewTask>>())
            .on_conflict((
                tasks::task_package_id,
                tasks::task_doc_id,
            ))
            .do_nothing()
            .execute(conn)?;

            tasks::table
            .select(
                (tasks::id, tasks::task_package_id, tasks::task_doc_id, tasks::task_type)
            )
            .filter(
                tasks::task_package_id.eq(task_package_id)
                .and(tasks::task_doc_id.eq_any(tasks.into_iter().map(|task| task.task_doc_id).collect::<Vec<Uuid>>()))
            ).load(conn)
        })
    }

    /// Fetches information about a task within a specific task collection.
    ///
    /// This function retrieves information about a task within a specified task collection
    ///  based on the provided
    /// task collection ID and task ID. It queries the database for the corresponding `GroupTask`
    ///  and returns the result.
    ///
    /// # Arguments
    ///
    /// * `task_package_id` - The unique identifier of the task collection to which the task belongs.
    /// * `task_doc_id` - The unique identifier of the task for which information is to be
    ///  fetched.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `GroupTask` struct, which includes the following
    ///  information:
    ///
    /// 1. `Uuid` - The unique identifier of the group task.
    /// 2. `Uuid` - The unique identifier of the group to which the task belongs.
    /// 3. `Uuid` - The unique identifier of the task.
    /// 4. `String` - The type of the task.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with the database
    ///  connection or if the specified
    /// task collection or task does not exist.
    ///
    /// # Example
    ///
    /// ```rust
    /// use diesel::prelude::*;
    ///
    /// let db_conn = PgRepo::establish_connection("database_url");
    /// let group_repo = db_conn.new_group_repo();
    /// let task_package_id = Uuid::new_v4();
    /// let task_doc_id = Uuid::new_v4(); // Replace with the actual task ID.
    ///
    /// match group_repo.fetch_task_from_group_task(&task_package_id, &task_doc_id) {
    ///     Ok(group_task) => {
    ///         println!("Group Task ID: {}, Group ID: {}, Task ID: {}, Task Type: {}",
    ///             group_task.id, group_task.task_package_id, group_task.task_doc_id, group_task.task_type);
    ///     },
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    ///
    /// In this example, the function is used to fetch information about a task
    ///  within a specific task collection and print the retrieved information.
    pub fn fetch_task_from_group_task(
        &self,
        task_package_id: &Uuid,
        task_doc_id: &Uuid,
    ) -> Result<Task, Error> {
        use crate::schema::tasks;

        let conn = 
            &mut self.pool.get().unwrap();

        tasks::table
            .select(
                (tasks::id, tasks::task_package_id, tasks::task_doc_id, tasks::task_type)
            )
            .filter(
                tasks::task_package_id.eq(task_package_id)
                .and(tasks::task_doc_id.eq(task_doc_id))
            )
            .first(conn)
    }
    
    /// Removes specified tasks from a task collection by updating their state to 'Deleted'.
    ///
    /// This function updates the state of tasks within a task collection to 'Deleted' based on the
    /// provided task IDs. It operates within a database transaction to ensure the atomicity of the
    /// update operation. After updating the state, it fetches the tasks with their new state for
    /// confirmation.
    ///
    /// # Parameters
    /// - `&self`: Reference to the current instance of the struct containing this method.
    /// - `task_package_id`: Reference to a UUID representing the ID of the task collection from which tasks are to be removed.
    /// - `task_doc_ids`: Reference to a vector of UUIDs representing the IDs of the tasks to be removed.
    ///
    /// # Returns
    /// `Result<Vec<CollectionTask>, Error>`: On success, returns a vector of `CollectionTask` structs representing the
    /// tasks that have been updated to 'Deleted'. On failure, returns an `Error`.
    ///
    /// # Errors
    /// This function might return an error if there are issues with database connectivity,
    /// transaction execution, or the update operation.
    ///
    /// # Example
    /// ```
    /// let task_doc_ids_to_remove = vec![/* list of task IDs */];
    /// let result = instance.remove_tasks_from_collection(&task_package_id, &task_doc_ids_to_remove);
    /// match result {
    ///     Ok(removed_tasks) => println!("Tasks removed: {:?}", removed_tasks),
    ///     Err(e) => println!("Error removing tasks: {}", e),
    /// }
    /// ```
    pub fn remove_tasks_from_package(
        &self,
        task_package_id: &Uuid,
        group_id: &Uuid,
        task_doc_ids: &Vec<Uuid>,
    ) -> Result<Vec<Task>, Error> {
        use crate::schema::tasks;
        use crate::schema::task_packages;

        let conn = 
            &mut self.pool.get().unwrap();

        conn.transaction(|conn| {
            let ids: Vec<Uuid> = tasks::table
                .inner_join(task_packages::table)
                .select(tasks::task_doc_id)
                .filter(
                    task_packages::group_id.eq(group_id)
                    .and(tasks::task_doc_id.eq_any(task_doc_ids))
                ).load(conn).unwrap();
            
            diesel::update(tasks::table)
            .filter(
                tasks::task_doc_id.eq_any(ids)
                .and(tasks::task_package_id.eq(task_package_id))
            )
            .set(
                tasks::state.eq(State::Deleted)
            ).execute(conn)?;

            tasks::table
            .select(
                (tasks::id, tasks::task_package_id, tasks::task_doc_id, tasks::task_type)
            )
            .filter(
                tasks::task_package_id.eq(task_package_id)
                .and(tasks::task_doc_id.eq_any(task_doc_ids))
                .and(tasks::state.eq(State::Deleted))
            ).load(conn)
        })
    }

    pub fn create_user_solution(
        &self,
        new_user_soltuion: &NewAnswer,
    ) -> Result<CreatedAnswer, Error> {
        use crate::schema::answers;

        let conn = 
            &mut self.pool.get().unwrap();
    
        conn.transaction(|conn| {
            diesel::insert_into(answers::table)
                .values(new_user_soltuion)
                .execute(conn)?;

            answers::table
                .select(
                    (answers::id, answers::solution_attempt_id, 
                    answers::answer_doc_id, answers::task_id,
                    answers::created_from)
                )
                .filter(
                    answers::solution_attempt_id.eq(new_user_soltuion.solution_attempt_id)
                    .and(answers::answer_doc_id.eq(new_user_soltuion.answer_doc_id))
                    .and(answers::task_id.eq(new_user_soltuion.task_id))
                )
                .first(conn)
        })
    }

    /// Asynchronously inserts an answer document ID into the associated user answer document.
    ///
    /// # Arguments
    ///
    /// * `answer_doc_id` - A reference to the UUID representing the answer document ID to be inserted.
    /// * `user_answer_doc_id` - A reference to the UUID representing the user answer document ID.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the number of affected rows (should be 1 if successful) or an `Error` if the operation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use your_crate_name::YourStructName;
    /// use uuid::Uuid;
    ///
    /// let your_struct_instance = YourStructName::new(); // Replace with your actual struct name and instance.
    /// let answer_doc_id = Uuid::new_v4();
    /// let user_answer_doc_id = Uuid::new_v4();
    ///
    /// match your_struct_instance.insert_answer_doc(&answer_doc_id, &user_answer_doc_id).await {
    ///     Ok(rows_affected) => {
    ///         println!("Inserted answer document ID into user answer document. Rows affected: {}", rows_affected);
    ///         // Handle the successful insertion
    ///     }
    ///     Err(err) => {
    ///         eprintln!("Error inserting answer document ID: {}", err);
    ///         // Handle the error
    ///     }
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if there is an issue retrieving the database connection or if the query execution fails.
    pub fn insert_answer_doc(
        &self,
        answer_doc_id: &Uuid,
        answer_id: &Uuid,
    ) -> Result<usize, Error> {
        use crate::schema::answers;

        let conn = 
            &mut self.pool.get().unwrap();

        diesel::update(answers::table)
            .set((
                answers::answer_doc_id.eq(answer_doc_id),
            )).filter(
                answers::answer_doc_id.eq(Uuid::default())
                .and(answers::id.eq(answer_id))
            )
            .execute(conn)
    }

/// Fetches the answer associated with the provided user answer document ID.
///
/// # Arguments
///
/// * `user_answer_doc_id` - A reference to the UUID representing the user answer document ID.
///
/// # Returns
///
/// Returns a `Result` containing the fetched `Answer` or an `Error` if the operation fails.
///
/// # Examples
///
/// ```
/// use your_crate_name::YourStructName;
/// use uuid::Uuid;
///
/// let your_struct_instance = YourStructName::new(); // Replace with your actual struct name and instance.
/// let user_answer_doc_id = Uuid::new_v4();
///
/// match your_struct_instance.fetch_answer(&user_answer_doc_id) {
///     Ok(answer) => {
///         println!("Fetched answer: {:?}", answer);
///         // Handle the fetched answer
///     }
///     Err(err) => {
///         eprintln!("Error fetching answer: {}", err);
///         // Handle the error
///     }
/// }
/// ```
///
/// # Panics
///
/// Panics if there is an issue retrieving the database connection or if the query execution fails.
    pub fn fetch_answer(
        &self,
        user_answer_doc_id: &Uuid,
        group_id: &Uuid,
    ) -> Result<Answer, Error> {
        use crate::schema::answers;
        use crate::schema::solution_attempts;
        use crate::schema::task_packages;
        
        let conn = 
            &mut self.pool.get().unwrap();

        answers::table
            .inner_join(
                solution_attempts::table
                .inner_join(task_packages::table)
            )
            .select(
                (answers::id, answers::correct, answers::solution_attempt_id, answers::answer_doc_id,
                answers::task_id, answers::state, answers::created_from)
            )
            .filter(
                answers::id.eq(user_answer_doc_id)
                .and(task_packages::group_id.eq(group_id))
            )
            .first(conn)
    }

    pub fn fetch_answers_from_solution_attempt(
        &self,
        solution_group_id: &Uuid
    ) -> Result<Vec<Answer>, Error> {
        use crate::schema::answers;
        use crate::schema::solution_attempts;

        let conn = 
            &mut self.pool.get().unwrap();

        answers::table
            .inner_join(
                solution_attempts::table
            )
            .select(
                (answers::id, answers::correct, answers::solution_attempt_id, answers::answer_doc_id,
                answers::task_id, answers::state, answers::created_from)
            )
            .filter(
                solution_attempts::id.eq(solution_group_id)
            ).load(conn)
    }

    /// Creates a new solution group for a user with associated tasks and returns it.
    ///
    /// This function creates a solution group for a specific user and task collection,
    /// optionally with a specified visibility. It then fetches tasks associated with the
    /// task collection, creating individual user solutions for each task, and associates
    /// them with the newly created solution group. The function operates within a database
    /// transaction to maintain consistency.
    ///
    /// # Parameters
    /// - `&self`: Reference to the current instance of the struct containing this method.
    /// - `user_id`: Reference to a UUID representing the user for whom the solution group is being created.
    /// - `task_package_id`: Reference to a UUID representing the ID of the task collection associated with the solution group.
    /// - `visibility`: Optional reference to a `Visibility` enum, specifying the visibility of the solution group.
    ///
    /// # Returns
    /// `Result<SolutionGroup, Error>`: On success, returns a `SolutionGroup` struct containing
    /// the newly created solution group, its state, and a list of associated solution task entries.
    /// On failure, returns an `Error`.
    ///
    /// # Errors
    /// This function might return an error if there are issues with database connectivity,
    /// transaction execution, data insertion, or task fetching.
    ///
    /// # Example
    /// ```
    /// let result = instance.create_solution_group(&user_id, &task_package_id, &Some(Visibility::Public));
    /// match result {
    ///     Ok(solution_group) => println!("Created solution group: {:?}", solution_group),
    ///     Err(e) => println!("Error creating solution group: {}", e),
    /// }
    /// ```
    pub fn create_solution_attempt(
        &self,
        user_id: &Uuid,
        task_package_id: &Uuid,
        group_id: &Uuid,
        visibility: &Option<Visibility>,
    ) -> Result<SolutionAttempt, Error> {
        use crate::schema::solution_attempts;
        use crate::schema::answers;

        let conn = 
            &mut self.pool.get().unwrap();

        conn.transaction(|conn| {
            let solution_attempt_id = Uuid::new_v4();

            diesel::insert_into(solution_attempts::table)
                .values(&NewSolutionAttempt {
                    id: solution_attempt_id,
                    user_id: user_id.to_owned(),
                    task_package_id: task_package_id.to_owned(),
                    visibility: visibility.to_owned(),
                })
                .execute(conn)?;

            let tasks = self.fetch_tasks_from_package(task_package_id, group_id, &None).unwrap();

            let mut solution_list = vec![];

            for task in tasks {

                let new_user_solution = &NewAnswer { 
                    solution_attempt_id: &solution_attempt_id, 
                    answer_doc_id: &Uuid::default(), 
                    task_id: &task.id, 
                    state: &AnswerState::Active,
                    created_from: user_id,
                };

                diesel::insert_into(answers::table)
                    .values(new_user_solution)
                    .execute(conn)?;

                let solution: CreatedAnswer = answers::table
                    .select(
                        (answers::id, answers::solution_attempt_id, 
                        answers::answer_doc_id, answers::task_id,
                        answers::created_from)
                    )
                    .filter(
                        answers::solution_attempt_id.eq(new_user_solution.solution_attempt_id)
                        .and(answers::answer_doc_id.eq(new_user_solution.answer_doc_id))
                        .and(answers::task_id.eq(new_user_solution.task_id))
                    )
                    .first(conn).unwrap();

                solution_list.push(AnswerEntry {
                    answer_id: solution.id,
                    answer_doc_id: solution.answer_doc_id,
                    task_id: task.id,
                    task_doc_id: task.task_doc_id,
                    task_type: task.task_type,
                });
            }

            let solution_group = solution_attempts::table
                .select((
                    solution_attempts::id,
                    solution_attempts::user_id,
                    solution_attempts::task_package_id,
                    solution_attempts::visibility,
                    solution_attempts::created_at,
                ))
                .filter(
                    solution_attempts::id.eq(solution_attempt_id)
                )
                .first(conn).unwrap();
            
            Ok(SolutionAttempt {
                state: AnswerState::Active,
                solution_attempt: solution_group,
                solution_list
            })
        })
    }

    /// Fetches a specific solution group and its associated solutions.
    ///
    /// This function retrieves a solution group based on the provided solution group ID and
    /// group ID. It also fetches the list of solutions associated with this group, excluding
    /// any solutions marked as 'Deleted'. Each solution's state is examined to determine the
    /// overall state of the solution group.
    ///
    /// # Parameters
    /// - `&self`: Reference to the current instance of the struct containing this method.
    /// - `solution_group_id`: Reference to a UUID representing the ID of the solution group to fetch.
    /// - `group_id`: Reference to a UUID representing the ID of the group to which the solution group belongs.
    ///
    /// # Returns
    /// `Result<SolutionGroup, Error>`: On success, returns a `SolutionGroup` struct that includes
    /// the solution group details, its overall state, and a list of associated solutions.
    /// On failure, returns an `Error`.
    ///
    /// # Errors
    /// This function might return an error if there are issues with database connectivity,
    /// transaction execution, or data retrieval.
    ///
    /// # Example
    /// ```
    /// let result = instance.fetch_solution_group(&solution_group_id, &group_id);
    /// match result {
    ///     Ok(solution_group) => println!("Fetched solution group: {:?}", solution_group),
    ///     Err(e) => println!("Error fetching solution group: {}", e),
    /// }
    /// ```
    pub fn fetch_solution_attempt(
        &self,
        solution_group_id: &Uuid,
        group_id: &Uuid,
    ) -> Result<SolutionAttempt, Error> {
        use crate::schema::solution_attempts;
        use crate::schema::answers;
        use crate::schema::tasks;
        use crate::schema::task_packages;

        let conn = 
            &mut self.pool.get().unwrap();

        let filter_query: 
            Box<dyn BoxableExpression<_, diesel::pg::Pg, SqlType = Bool>> =
            Box::new(
                solution_attempts::id.eq(solution_group_id)
                .and(task_packages::group_id.eq(group_id))
            );

        conn.transaction(|conn| {
            let solution_group = solution_attempts::table
                .inner_join(task_packages::table)
                .select((
                    solution_attempts::id,
                    solution_attempts::user_id,
                    solution_attempts::task_package_id,
                    solution_attempts::visibility,
                    solution_attempts::created_at,
                ))
                .filter(
                    filter_query
                )
                .first(conn).unwrap();

            let solution_list: Vec<(Uuid, Uuid, Uuid, Uuid, String, AnswerState)> = answers::table
                .inner_join(tasks::table)
                .select((
                    answers::id,
                    answers::answer_doc_id,
                    tasks::id,
                    tasks::task_doc_id,
                    tasks::task_type,
                    answers::state,
                ))
                .filter(
                    answers::solution_attempt_id.eq(solution_group_id)
                    .and(answers::state.ne(AnswerState::Deleted))
                )
                .load(conn).unwrap();

            let mut state = AnswerState::Done;
            
            let ret_soltuion_list = solution_list.into_iter().map(|entry| {
                if entry.5 == AnswerState::Active {
                    state = AnswerState::Active;
                }
                AnswerEntry {
                    answer_id: entry.0,
                    answer_doc_id: entry.1,
                    task_id: entry.2,
                    task_doc_id: entry.3,
                    task_type: entry.4,
                }
            }).collect();

            Ok(SolutionAttempt { 
                solution_attempt: solution_group, 
                state,
                solution_list: ret_soltuion_list,
            })
        })
    }

    /// Fetches solution groups for a specific user, task collection, and group, filtered by visibility.
    ///
    /// This function retrieves solution groups that match the given parameters. It performs
    /// a filtered query on the 'solution_attempts' table, optionally filtering by the
    /// visibility status. For each solution group, it also fetches the solution state,
    /// returning a collection of solution groups along with their corresponding states.
    ///
    /// # Parameters
    /// - `&self`: Reference to the current instance of the struct containing this method.
    /// - `user_id`: Reference to a UUID representing the user ID to filter the solution groups.
    /// - `task_package_id`: Reference to a UUID representing the task collection ID.
    /// - `group_id`: Reference to a UUID representing the group ID.
    /// - `visibility`: Optional reference to a `Visibility` enum, specifying the visibility
    ///   filter for the solution groups.
    ///
    /// # Returns
    /// `Result<Vec<(CreatedSolutionUserGroup, SolutionState)>, Error>`: On success, returns a
    /// vector of tuples. Each tuple contains a `CreatedSolutionUserGroup` and its corresponding
    /// `SolutionState`. On failure, returns an `Error`.
    ///
    /// # Errors
    /// This function might return an error if there are issues with database connectivity,
    /// transaction execution, data retrieval, or if visibility filtering fails.
    ///
    /// # Example
    /// ```
    /// let result = instance.fetch_solution_groups(
    ///     &user_id,
    ///     &task_package_id,
    ///     &group_id,
    ///     &Some(Visibility::Public),
    /// );
    /// match result {
    ///     Ok(solution_groups) => {
    ///         for (group, state) in solution_groups {
    ///             println!("Solution Group: {:?}, State: {:?}", group, state);
    ///         }
    ///     },
    ///     Err(e) => println!("Error fetching solution groups: {}", e),
    /// }
    /// ```
    pub fn fetch_solution_attempts(
        &self,
        user_id: &Uuid,
        task_package_id: &Uuid,
        group_id: &Uuid,
        visibility: &Option<Visibility>,
    ) -> Result<Vec<(CreatedSolutionAttempt, AnswerState)>, Error> {
        use crate::schema::solution_attempts;
        use crate::schema::answers;
        use crate::schema::task_packages;

        let conn = 
            &mut self.pool.get().unwrap();
        
        let mut filter_query: 
            Box<dyn BoxableExpression<_, diesel::pg::Pg, SqlType = Bool>> =
            Box::new(
                solution_attempts::user_id.eq(user_id)
                .and(solution_attempts::task_package_id.eq(task_package_id))
                .and(task_packages::group_id.eq(group_id))
            );

        if visibility.is_some() {
            filter_query = Box::new(filter_query.and(solution_attempts::visibility.eq(visibility.as_ref().unwrap())));
        }

        conn.transaction(|conn| {
            let solution_attempts: Vec<CreatedSolutionAttempt> = solution_attempts::table
            .inner_join(task_packages::table)
            .select((
                solution_attempts::id,
                solution_attempts::user_id,
                solution_attempts::task_package_id,
                solution_attempts::visibility,
                solution_attempts::created_at,
            ))
            .filter(
                filter_query
            ).load(conn).unwrap();

            let mut solution_attempts_with_state: Vec<(CreatedSolutionAttempt, AnswerState)> = vec![];
            for solution_group in solution_attempts {
                let state = answers::table
                    .inner_join(solution_attempts::table)
                    .select(answers::state)
                    .filter(
                        solution_attempts::id.eq(solution_group.id)
                        .and(answers::state.eq(AnswerState::Active))
                    ).first(conn).unwrap_or(AnswerState::Done);

                    solution_attempts_with_state.push((solution_group, state));
            }

            Ok(solution_attempts_with_state)
        })
    }

    pub fn create_task_package(
        &self,
        name: &str,
        group_id: &Uuid,
        task_package_type: &Option<TaskPackageType>,
        tasks: &Vec<NewTempTask>
    ) -> Result<CreatedTaskPackage, Error>  {
        use crate::schema::task_packages;
        use crate::schema::tasks;

        let conn = 
        &mut self.pool.get().unwrap();

        conn.transaction(|conn| {
            let task_package_id = Uuid::new_v4();

            diesel::insert_into(task_packages::table)
                .values(&NewTaskPackage {
                    id: task_package_id,
                    group_id: group_id.to_owned(),
                    name: name.to_string(),
                    task_package_type: task_package_type.to_owned(),
                })
                .execute(conn)?;

            diesel::insert_into(tasks::table)
                .values(tasks.into_iter().map(|task| NewTask {
                    task_package_id: &task_package_id,
                    task_doc_id: &task.task_doc_id,
                    task_type: &task.task_type,
                }).collect::<Vec<NewTask>>())
                .on_conflict((
                    tasks::task_package_id,
                    tasks::task_doc_id,
                ))
                .do_nothing()
                .execute(conn)?;

            task_packages::table
                .select((
                    task_packages::id, task_packages::name,
                    task_packages::group_id, task_packages::task_package_type,
                    task_packages::created_at
                ))
                .filter(
                    task_packages::id.eq(task_package_id)
                )
                .first(conn)
        })
    }

    pub fn finish_solution_attempt(
        &self,
        solution_group_id: &Uuid,
        answer_correct_state: &HashMap<Uuid, bool>,
    ) -> Result<usize, Error>{
        use crate::schema::answers;
        use crate::schema::solution_attempts;

        let conn = 
            &mut self.pool.get().unwrap();
            
        conn.transaction(|conn| {
            let answer_doc_ids: Vec<Uuid> = answers::table
                .inner_join(solution_attempts::table)
                .filter(
                    solution_attempts::id.eq(solution_group_id)
                    .and(answers::state.eq(AnswerState::Active))
                )
                .select(answers::id).load(conn).unwrap();

            let mut amount_updated = 0;
            for answer_id in answer_doc_ids {
                amount_updated += diesel::update(answers::table)
                .set((
                    answers::state.eq(AnswerState::Done),
                    answers::correct.eq(answer_correct_state.get(&answer_id).unwrap())
                ))
                .filter(answers::id.eq(answer_id))
                .execute(conn).unwrap();
            }
            Ok(amount_updated)
        })
    }

    pub fn fetch_task_package_user_statistic(
        &self,
        group_id: &Uuid,
        user_id: &Uuid,
        task_package_id: &Uuid,
        visibility: &Option<Visibility>,
        task_types: &Option<Vec<String>>,
    ) -> Result<Vec<TaskPackageUserStatisticValue>, Error> {
        use crate::schema::task_packages;
        use crate::schema::answers;
        use crate::schema::solution_attempts;
        use crate::schema::tasks;

        let conn = 
            &mut self.pool.get().unwrap();

        let mut filter_query: 
            Box<dyn BoxableExpression<_, diesel::pg::Pg, SqlType = Bool>> =
            Box::new(
                solution_attempts::user_id.eq(user_id)
                .and(task_packages::id.eq(task_package_id))
                .and(task_packages::group_id.eq(group_id))
            );

        if visibility.is_some() {
            filter_query = Box::new(
                filter_query.and(solution_attempts::visibility.eq(visibility.as_ref().unwrap()))
            );
        }

        if task_types.is_some() {
            filter_query = Box::new(
                filter_query.and(
                    tasks::task_type.eq_any(task_types.as_ref().unwrap())
                )
            );
        }

        conn.transaction(|conn| {
            let statistics: Vec<(Uuid, NaiveDateTime, bool, Uuid)> = answers::table
                .inner_join(
                    solution_attempts::table
                    .inner_join(task_packages::table)
                )
                .inner_join(tasks::table)
                .select((
                    solution_attempts::id,
                    solution_attempts::updated_at,
                    answers::correct,
                    answers::id,
                ))
                .filter(filter_query)
                .load(conn).unwrap();

            
            let mut user_statistics: Vec<TaskPackageUserStatisticValue> = vec![];
            for (key, group) in &statistics.into_iter().group_by(|elm| (elm.0, elm.1)) {
                user_statistics.push(TaskPackageUserStatisticValue {
                    amount_correct: group.filter(|v| v.2 == true).count(),
                    date: key.1.and_utc(),
                    solution_attempt_id: key.0,
                })
            }

            Ok(user_statistics)
        })
    }
}