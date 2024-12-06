use mongodb::Database;
use poem::web::Data;
use poem_openapi::{
    param::{Path, Query},
    payload::Json,
    OpenApi,
};

use poem_openapi::Tags;

#[derive(Tags)]
pub(crate) enum APITags {
    Check,
    History,
}

pub(crate) struct MonitorAPI;

#[OpenApi]
impl MonitorAPI {
    /// Create new check
    #[oai(method = "post", path = "/", tag = APITags::Check)]
    async fn create_check(&self, Data(database): Data<&Database>) {
        todo!()
    }

    /// Update a check
    #[oai(method = "put", path = "/", tag = APITags::Check)]
    async fn update_check(&self, Data(database): Data<&Database>) {
        todo!()
    }

    /// Read check history
    #[oai(method = "get", path = "/:check_id", tag = APITags::History)]
    async fn read_history(&self, Data(database): Data<&Database>) {
        todo!()
    }

    /// Delete check history
    #[oai(method = "delete", path = "/", tag = APITags::History)]
    async fn delete_history(&self, Data(database): Data<&Database>) {
        todo!()
    }
}
