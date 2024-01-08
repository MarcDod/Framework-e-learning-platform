// Documentation was created by ChatGPT
use diesel::{prelude::*, dsl::exists, result::Error, sql_types::Bool, upsert::excluded};
use uuid::Uuid;

use crate::{models::{users::{NewUser, UserInfo, UserPassword, User}, util::{State, AccessType}, groups::{RolePermission, NewUserPermission}, permissions::{RoleAccesType, NewUserAccessType}}};

use super::postgres::DBPool;

#[derive(Clone)]
pub struct UsersRepo {
    pool: DBPool
}

pub const CREATED_USER_ROLE_KEY: &str = "created_user";

impl UsersRepo {

    pub fn new(pool: DBPool) -> Self {
        UsersRepo { pool }
    }

    /// Fetches permissions associated with a specific role.
    ///
    /// This function retrieves a list of permissions for a given role, identified by its key.
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
    /// This function might return an error if there are issues with database connectivity or the
    /// execution of the query.
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

    /// Checks the existence of a user with a given email in the database.
    ///
    /// This function queries the database to determine whether a user with the specified email address exists.
    ///
    /// # Arguments
    ///
    /// * `user_email` - The email address to check for existence in the database.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `bool` indicating whether a user with the given email exists (`true`) or not (`false`).
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with the database connection or if an error occurs during the query.
    ///
    /// # Example
    ///
    /// ```rust
    /// use diesel::prelude::*;
    ///
    /// let db_conn = PgRepo::establish_connection("database_url");
    /// let user_repo = db_conn.new_user_repo();
    /// let user_email = "user@example.com".to_string(); // Replace with the actual email address.
    ///
    /// match user_repo.exist_user_with_email(user_email) {
    ///     Ok(user_exists) => {
    ///         if user_exists {
    ///             println!("User with email exists!");
    ///         } else {
    ///             println!("User with email does not exist.");
    ///         }
    ///     },
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    ///
    /// In this example, the function is used to check whether a user with a specific email address exists in the database and prints the result.
    pub fn exist_user_with_email(&self, user_email: String) -> Result<bool, Error> {
        use crate::schema::users;

        let conn = &mut self.pool.get().unwrap();

        let sub_query = users::table
            .select(0.into_sql::<diesel::sql_types::Integer>())
            .filter(
                users::email.eq(&user_email)
            );

        diesel::select(exists(sub_query)).get_result::<bool>(conn)
    }

    /// Fetches user information by their unique identifier (UUID) from the database.
    ///
    /// This function retrieves user information, including user ID, name, and email, based on the provided user ID.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The unique identifier (UUID) of the user whose information is to be fetched.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `UserInfo` struct representing the user's information if found.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with the database connection or if the user with the specified ID is not found.
    ///
    /// # Example
    ///
    /// ```rust
    /// use diesel::prelude::*;
    ///
    /// let db_conn = PgRepo::establish_connection("database_url");
    /// let user_repo = db_conn.new_user_repo();
    /// let user_id = Uuid::new_v4(); // Replace with the actual UUID of the user.
    ///
    /// match user_repo.fetch_user_by_id(user_id) {
    ///     Ok(user_info) => {
    ///         println!("User ID: {}", user_info.id);
    ///         println!("User Name: {}", user_info.name);
    ///         println!("User Email: {}", user_info.email);
    ///     },
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    ///
    /// In this example, the function is used to fetch user information by their ID from the database and prints the retrieved user details.
    pub fn fetch_user_by_id(&self, user_id: Uuid) -> Result<UserInfo, Error> {
        use crate::schema::users;

        let conn = &mut self.pool.get().unwrap();

        users::table
            .select((users::id, users::name, users::email))
            .filter(
                users::id.eq(&user_id)
            )
            .first(conn)
    }

    /// Fetches the active user's unique identifier (UUID) based on their email from the database.
    ///
    /// This function retrieves the user ID for an active user by their email address.
    ///
    /// # Arguments
    ///
    /// * `user_email` - The email address of the user for whom to fetch the ID.
    ///
    /// # Returns
    ///
    /// A `Result` containing the UUID representing the active user's unique identifier.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with the database connection, if the user with the specified email is not found, or if the user is not in an active state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use diesel::prelude::*;
    ///
    /// let db_conn = PgRepo::establish_connection("database_url");
    /// let user_repo = db_conn.new_user_repo();
    /// let user_email = "example@example.com"; // Replace with the actual email address.
    ///
    /// match db_conn.fetch_active_user_id_by_email(user_email) {
    ///     Ok(user_id) => {
    ///         println!("Active User ID: {}", user_id);
    ///     },
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    ///
    /// In this example, the function is used to fetch the UUID of an active user by their email address from the database and prints the retrieved user ID.
    pub fn fetch_active_user_id_by_email(&self, user_email: &str) -> Result<Uuid, Error> {
        use crate::schema::users;

        let conn = &mut self.pool.get().unwrap();

        users::table
            .select(users::id)
            .filter(
                users::email.eq(user_email)
                .and(users::state.eq(State::Active))
            )
            .first(conn)
    }

    /// Fetches the active user's password information based on their email from the database.
    ///
    /// This function retrieves the user's ID and password for an active user by their email address.
    ///
    /// # Arguments
    ///
    /// * `user_email` - The email address of the user for whom to fetch the password information.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `UserPassword` struct representing the active user's ID and password.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with the database connection, if the user with the specified email is not found, or if the user is not in an active state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use diesel::prelude::*;
    ///
    /// let db_conn = PgRepo::establish_connection("database_url");
    /// let user_repo = db_conn.new_user_repo();
    /// let user_email = "example@example.com"; // Replace with the actual email address.
    ///
    /// match db_conn.fetch_active_user_password_by_email(user_email.to_string()) {
    ///     Ok(user_password) => {
    ///         println!("User ID: {}", user_password.id);
    ///         println!("User Password: {}", user_password.password);
    ///     },
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    ///
    /// In this example, the function is used to fetch the ID and password of an active user by their email address from the database and prints the retrieved user ID and password.
    pub fn fetch_active_user_password_by_email(&self, user_email: String) -> Result<UserPassword, Error> {
        use crate::schema::users;

        let conn = &mut self.pool.get().unwrap();
        
        users::table
            .select((users::id, users::password))
            .filter(
                users::email.eq(&user_email)
                .and(users::state.eq(State::Active))
            )
            .first(conn)
    }

    /// Creates a new user and associated permissions in the database.
    ///
    /// This function inserts a new user into the database and assigns the specified permissions to the user. It returns the created user's information.
    ///
    /// # Arguments
    ///
    /// * `new_user` - The `NewUser` struct containing the details of the user to be created.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `UserInfo` struct representing the information of the created user.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with the database connection or if there is an error during the user creation process.
    ///
    /// # Example
    ///
    /// ```rust
    /// use diesel::prelude::*;
    ///
    /// // Create a new user with the necessary details.
    /// let new_user = NewUser {
    ///     name: "John Doe",
    ///     email: "john.doe@example.com",
    ///     // Add other user details as needed.
    /// };
    ///
    /// let db_conn = PgRepo::establish_connection("database_url");
    /// let user_repo = db_conn.new_user_repo();
    ///
    /// match db_conn.create_user(&new_user) {
    ///     Ok(created_user) => {
    ///         println!("User ID: {}", created_user.id);
    ///         println!("User Name: {}", created_user.name);
    ///         println!("User Email: {}", created_user.email);
    ///     },
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    ///
    /// In this example, the function is used to create a new user in the database with the specified details and print the user information. 
    pub fn create_user(&self, new_user: &NewUser) -> Result<UserInfo, Error> {
        use crate::schema::users;
        use crate::schema::role_access_types;
        use crate::schema::user_access_types;

        let conn = &mut self.pool.get().unwrap();

        let role_permissions = match self.fetch_role_permissions(CREATED_USER_ROLE_KEY) {
            Ok(v) => v,
            Err(err) => return Err(err),
        };

        conn.transaction(|conn | {
            diesel::insert_into(users::table)
                .values(new_user)
                .execute(conn)?;

            let user: UserInfo = match users::table
                .select((users::id, users::name, users::email))
                .filter(users::email.eq(&new_user.email))
                .first(conn){
                    Ok(v) => v,
                    Err(err) => return Err(err),
                };

            for role_permission in role_permissions {

                let user_permission_id = Uuid::new_v4();

                let access_types: Vec<RoleAccesType> = role_access_types::table
                    .select((
                        role_access_types::access_type,
                        role_access_types::set_permission,
                        role_access_types::set_set_permission,
                        role_access_types::permission,
                    )).filter(
                        role_access_types::role_permission_id.eq(role_permission.0)
                    ).load(conn).unwrap();

                diesel::insert_into(user_access_types::table)
                    .values(access_types.into_iter().map(|access_type| NewUserAccessType {
                        access_type: access_type.access_type,
                        permission: Some(access_type.permission),
                        set_permission: Some(access_type.set_permission),
                        set_set_permission: Some(access_type.set_set_permission),
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
            
            Ok(user)
        })
    }

    #[cfg(test)]
    pub fn delete_user(&self, user_id: &Uuid) -> Result<usize, Error> {
        use crate::schema::users;

        let conn = &mut self.pool.get().unwrap();

        diesel::delete(users::table)
            .filter(users::id.eq(user_id))
            .execute(conn)
    }

    #[cfg(test)]
    pub fn fetch_amount_user(&self) -> Result<i64, Error> {
        use crate::schema::users;

        let conn = 
            &mut self.pool.get().unwrap();
        
        users::table
            .count()
            .get_result(conn)
    }

    #[cfg(test)]
    pub fn fetch_user(&self, user_id: &Uuid) -> Result<User, Error> {
        use crate::schema::users;

        let conn = 
            &mut self.pool.get().unwrap();

        users::table
            .select(
                (users::id, users::name, 
                users::email, users::password, users::state,
                users::created_at, users::updated_at)
            )
            .filter(
                users::id.eq(user_id)
            )
            .first(conn)
    }
}