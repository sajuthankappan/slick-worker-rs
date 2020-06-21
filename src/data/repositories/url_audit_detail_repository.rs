use crate::models::UrlAuditDetail;
use wread_data_mongodb::crud_repository;
use wread_data_mongodb::mongodb::{results::InsertOneResult, Database, error::Error};

const COLLECTION_NAME: &str = "urlAuditDetails";

pub async fn add(audit_detail: &UrlAuditDetail, db: &Database) -> Result<InsertOneResult, Error> {
    crud_repository::add(audit_detail, COLLECTION_NAME, db).await
}
