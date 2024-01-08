use diesel::dsl::count_star;
use diesel::r2d2::{self, ConnectionManager};
use diesel::{prelude::*, result::Error};

use uuid::Uuid;

use crate::models::util::State;

use super::{users::UsersRepo, group::GroupRepo};
use super::permissions::PermissionsRepo;

pub type DBPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct PgRepo {
    pool: DBPool,
}

impl PgRepo {
    pub fn establish_connection(database_url: String) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(&database_url);
        let pool = r2d2::Pool::builder()
            .max_size(8)
            .build(manager)
            .expect("Failed to create pool.");

        PgRepo { pool }
    }

    pub fn new_user_repo(&self) -> UsersRepo {
        UsersRepo::new(self.pool.clone())
    }

    pub fn new_permissions_repo(&self) -> PermissionsRepo {
        PermissionsRepo::new(self.pool.clone())
    }

    pub fn new_group_repo(&self) -> GroupRepo {
        GroupRepo::new(self.pool.clone())
    }

    pub fn user_exists(&self, user_id: &Uuid) -> bool {
        use crate::schema::users;
        
        let conn = 
            &mut self.pool.get().unwrap();
        
        let result: Result<i64, Error> = users::table
            .select(count_star())
            .filter(
                users::id.eq(user_id)
                .and(users::state.eq(State::Active))    
            )
            .first(conn);

        result.unwrap_or(0) == 1
    }

    #[cfg(test)]
    pub fn clear_db(&self) {
        use crate::schema::users;
        use crate::schema::user_permissions;
        use crate::schema::group_members;
        use crate::schema::groups;
        use crate::schema::ressources;
        use crate::schema::roles;
        use crate::schema::role_permissions;
        use crate::schema::ressource_access_types;
        use crate::schema::answers;
        use crate::schema::group_ancestors;
        use crate::schema::tasks;
        use crate::schema::user_access_types;
        use crate::schema::task_packages;
        use crate::schema::solution_attempts;
        use crate::schema::role_access_types;

        let conn = &mut self.pool.get().unwrap();

        diesel::delete(role_access_types::table).execute(conn).unwrap();
        diesel::delete(role_permissions::table).execute(conn).unwrap();
        diesel::delete(roles::table).execute(conn).unwrap();
        diesel::delete(user_access_types::table).execute(conn).unwrap();
        diesel::delete(ressource_access_types::table).execute(conn).unwrap();
        diesel::delete(user_permissions::table).execute(conn).unwrap();
        diesel::delete(group_ancestors::table).execute(conn).unwrap();
        diesel::delete(group_members::table).execute(conn).unwrap();
        diesel::delete(answers::table).execute(conn).unwrap();
        diesel::delete(solution_attempts::table).execute(conn).unwrap();
        diesel::delete(ressources::table).execute(conn).unwrap();
        diesel::delete(tasks::table).execute(conn).unwrap();
        diesel::delete(task_packages::table).execute(conn).unwrap();
        diesel::delete(groups::table).execute(conn).unwrap();
        diesel::delete(users::table).execute(conn).unwrap();
    }
}