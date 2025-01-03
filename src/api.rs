use bson::doc;
use futures::TryStreamExt;

use bson::oid::ObjectId;
use mongodb::Database;
use poem::web::Data;
use poem_openapi::param::Path;
use poem_openapi::{payload::Json, OpenApi};

use poem_openapi::Tags;

use crate::models::{Check, CheckHistory, Error, HTTPMethod, NewCheck, UpdateCheck};

#[derive(Tags)]
pub(crate) enum APITags {
    Check,
    History,
}

pub(crate) struct MonitorAPI;

mod responses {
    use crate::models::{Check, CheckHistory, Error};
    use poem_openapi::{payload::Json, ApiResponse};

    #[derive(ApiResponse)]
    pub(crate) enum ReadChecksResponse {
        #[oai(status = 201)]
        Success(Json<Vec<Check>>),

        #[oai(status = 500)]
        InternalServerError(Json<Error>),
    }

    #[derive(ApiResponse)]
    pub(crate) enum ReadCheckResponse {
        #[oai(status = 201)]
        Success(Json<Check>),

        #[oai(status = 404)]
        NotFound(Json<Error>),

        #[oai(status = 500)]
        InternalServerError(Json<Error>),
    }

    #[derive(ApiResponse)]
    pub(crate) enum CreateCheckResponse {
        #[oai(status = 201)]
        Success(Json<Check>),

        #[oai(status = 400)]
        BadRequest(Json<Error>),
    }

    #[derive(ApiResponse)]
    pub(crate) enum UpdateCheckResponse {
        #[oai(status = 204)]
        Success,

        #[oai(status = 404)]
        NotFound(Json<Error>),

        #[oai(status = 400)]
        BadRequest(Json<Error>),

        #[oai(status = 500)]
        InternalServerError(Json<Error>),
    }

    #[derive(ApiResponse)]
    pub(crate) enum DeleteCheckResponse {
        #[oai(status = 204)]
        Success,

        #[oai(status = 404)]
        NotFound(Json<Error>),

        #[oai(status = 500)]
        InternalServerError(Json<Error>),
    }

    #[derive(ApiResponse)]
    pub(crate) enum ReadHistoryResponse {
        #[oai(status = 201)]
        Success(Json<Vec<CheckHistory>>),

        #[oai(status = 500)]
        InternalServerError(Json<Error>),
    }

    #[derive(ApiResponse)]
    pub(crate) enum DeleteHistoryResponse {
        #[oai(status = 204)]
        Success,

        #[oai(status = 404)]
        NotFound(Json<Error>),

        #[oai(status = 500)]
        InternalServerError(Json<Error>),
    }
}

#[OpenApi]
impl MonitorAPI {
    /// Read all checks
    #[oai(method = "get", path = "/", tag = APITags::Check)]
    async fn read_checks(&self, Data(database): Data<&Database>) -> responses::ReadChecksResponse {
        let collection = database.collection::<Check>("checks");
        let checks = collection.find(doc! {}).await;
        if checks.is_err() {
            let err = unsafe { checks.unwrap_err_unchecked() };
            return responses::ReadChecksResponse::InternalServerError(Json(
                Error::internal_server_error(err.to_string()),
            ));
        }

        responses::ReadChecksResponse::Success(Json(checks.unwrap().try_collect().await.unwrap()))
    }

    /// Read one check
    #[oai(method = "get", path = "/:check_id", tag = APITags::Check)]
    async fn read_check(
        &self,
        Data(database): Data<&Database>,
        Path(check_id): Path<ObjectId>,
    ) -> responses::ReadCheckResponse {
        let collection = database.collection::<Check>("checks");
        let check = collection.find_one(doc! {"_id": check_id}).await;
        if check.is_err() {
            let err = unsafe { check.unwrap_err_unchecked() };
            return responses::ReadCheckResponse::InternalServerError(Json(
                Error::internal_server_error(err.to_string()),
            ));
        }

        if let Some(check) = check.unwrap() {
            return responses::ReadCheckResponse::Success(Json(check));
        } else {
            return responses::ReadCheckResponse::NotFound(Json(Error::not_found(format!(
                "Check not found with id '{check_id}'"
            ))));
        }
    }

    /// Create new check
    #[oai(method = "post", path = "/", tag = APITags::Check)]
    async fn create_check(
        &self,
        Data(database): Data<&Database>,
        Json(new_check): Json<NewCheck>,
    ) -> responses::CreateCheckResponse {
        let check = Check::from_new(new_check);

        if check.expected_body.is_some() && check.method == HTTPMethod::HEAD {
            return responses::CreateCheckResponse::BadRequest(Json(Error::bad_request(
                "Expected body parameter is only allowed with GET requests.".to_string(),
            )));
        }

        let collection = database.collection::<Check>("checks");
        let result = collection.insert_one(check.clone()).await;
        result.map_or_else(
            |e| responses::CreateCheckResponse::BadRequest(Json(Error::bad_request(e.to_string()))),
            |_| responses::CreateCheckResponse::Success(Json(check)),
        )
    }

    /// Update check
    #[oai(method = "put", path = "/:check_id", tag = APITags::Check)]
    async fn update_check(
        &self,
        Data(database): Data<&Database>,
        Path(check_id): Path<ObjectId>,
        Json(update): Json<UpdateCheck>,
    ) -> responses::UpdateCheckResponse {
        let mut update_doc = doc! {
            "updated_at": chrono::Utc::now()
                .to_string(),
        };
        if let Some(frequency) = update.frequency {
            update_doc.insert("frequency", frequency.to_string());
        }

        if let Some(url) = update.url {
            update_doc.insert("url", url);
        }

        if let Some(method) = update.method {
            update_doc.insert("method", method.to_string());
        }

        if let Some(expected_body) = update.expected_body {
            if let Ok(bson_expected_body) = bson::to_bson(&expected_body) {
                update_doc.insert("expected_body", bson_expected_body);
            } else {
                return responses::UpdateCheckResponse::BadRequest(Json(Error::bad_request(
                    "Failed to serialize expected_body".to_string(),
                )));
            }
        }

        if let Some(hook) = update.hook {
            update_doc.insert("hook", bson::to_bson(&hook).unwrap());
        }

        let mut filter = doc! {"_id": check_id};
        if update_doc.contains_key("expected_body") {
            if let Some(updating_method) = update_doc.get("method") {
                if updating_method.as_str().unwrap() == "HEAD" {
                    // If we are updating both the method and the expected body,
                    // ensure that the method is GET, otherwise the ping will fail
                    return responses::UpdateCheckResponse::BadRequest(Json(Error::bad_request(
                        "Expected body parameter is only allowed with GET requests.".to_string(),
                    )));
                }
            } else {
                // If we are providing an expected body, ensure that the method
                // used to ping the service is GET, otherwise the ping will fail
                filter.insert("method", HTTPMethod::GET.to_string());
            }
        }

        let collection = database.collection::<Check>("checks");
        let update = collection
            .update_one(
                filter,
                doc! {
                    "$set":  update_doc
                },
            )
            .await;

        if let Ok(update) = update {
            if update.matched_count > 0 {
                responses::UpdateCheckResponse::Success
            } else {
                responses::UpdateCheckResponse::NotFound(Json(Error::not_found(format!(
                    "Check not found with id '{check_id}'"
                ))))
            }
        } else {
            let err = unsafe { update.unwrap_err_unchecked() };
            responses::UpdateCheckResponse::InternalServerError(Json(Error::internal_server_error(
                err.to_string(),
            )))
        }
    }

    /// Delete check
    #[oai(method = "delete", path = "/:check_id", tag = APITags::History)]
    async fn delete_check(
        &self,
        Data(database): Data<&Database>,
        Path(check_id): Path<ObjectId>,
    ) -> responses::DeleteCheckResponse {
        let collection = database.collection::<Check>("checks");
        let delete = collection.delete_one(doc! {"_id": check_id}).await;
        if let Ok(delete) = delete {
            if delete.deleted_count > 0 {
                responses::DeleteCheckResponse::Success
            } else {
                responses::DeleteCheckResponse::NotFound(Json(Error::not_found(format!(
                    "Check not found with id '{check_id}'"
                ))))
            }
        } else {
            let err = unsafe { delete.unwrap_err_unchecked() };
            responses::DeleteCheckResponse::InternalServerError(Json(Error::internal_server_error(
                err.to_string(),
            )))
        }
    }

    /// Read check history
    #[oai(method = "get", path = "/:check_id/history", tag = APITags::History)]
    async fn read_history(
        &self,
        Data(database): Data<&Database>,
        Path(check_id): Path<ObjectId>,
    ) -> responses::ReadHistoryResponse {
        let collection = database.collection::<CheckHistory>("checks_history");
        let checks = collection.find(doc! {"check_id": check_id}).await;
        if checks.is_err() {
            let err = unsafe { checks.unwrap_err_unchecked() };
            return responses::ReadHistoryResponse::InternalServerError(Json(
                Error::internal_server_error(err.to_string()),
            ));
        }

        responses::ReadHistoryResponse::Success(Json(checks.unwrap().try_collect().await.unwrap()))
    }

    /// Delete check history
    #[oai(method = "delete", path = "/:check_id/history", tag = APITags::History)]
    async fn delete_history(
        &self,
        Data(database): Data<&Database>,
        Path(check_id): Path<ObjectId>,
    ) -> responses::DeleteHistoryResponse {
        let collection = database.collection::<CheckHistory>("checks_history");
        let delete = collection.delete_one(doc! {"check_id": check_id}).await;
        if let Ok(delete) = delete {
            if delete.deleted_count > 0 {
                responses::DeleteHistoryResponse::Success
            } else {
                responses::DeleteHistoryResponse::NotFound(Json(Error::not_found(format!(
                    "Check not found with id '{check_id}'"
                ))))
            }
        } else {
            let err = unsafe { delete.unwrap_err_unchecked() };
            responses::DeleteHistoryResponse::InternalServerError(Json(
                Error::internal_server_error(err.to_string()),
            ))
        }
    }
}
