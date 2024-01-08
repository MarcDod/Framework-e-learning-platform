use actix_web::web::{ServiceConfig, self, Data};

use crate::{handlers::auth::auth, repository::{postgres::PgRepo, users::UsersRepo, permissions::PermissionsRepo}};

use super::{groups::groups, tasks::tasks, user::user, users::users, ressources::ressources};

pub fn config(cfg: &mut ServiceConfig, pgdb: PgRepo) {
    let user_db = pgdb.new_user_repo();
    let permission_db = pgdb.new_permissions_repo();

    cfg.service(
        web::scope("/api")
            .app_data(Data::<UsersRepo>::new(user_db.clone()))
            .app_data(Data::<PermissionsRepo>::new(permission_db.clone()))
            .configure(|cfg| user::config(cfg, pgdb.clone()))
            .configure(users::config)
            .configure(|cfg| groups::config(cfg, pgdb.clone()))
            .configure(tasks::config)
            .configure(auth::config)
            .configure(ressources::config)
    );
}