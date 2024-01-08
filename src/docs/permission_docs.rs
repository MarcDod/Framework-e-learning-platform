use utoipa::OpenApi;

use crate::handlers;
use crate::models::permissions::{
    RessourceAndAccessTypesListWithCount,
    RessourcesPagingSchema,
    RessourceWithAccessTypes,
    UserAccessType,
};

use crate::models::util::AccessType;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::ressources::ressources::fetch_ressources,
    ), 
    components(schemas(
        RessourceAndAccessTypesListWithCount,
        RessourcesPagingSchema,
        RessourceWithAccessTypes,
        UserAccessType,
        AccessType
    )), 
    tags(
        (name="permission", description = "Ressource endpoints"),
    ), 
)]
pub struct ApiDoc;