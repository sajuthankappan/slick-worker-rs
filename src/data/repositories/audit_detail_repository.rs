use crate::models::AuditDetail;
use wread_data_mongodb::crud_repository;
use wread_data_mongodb::mongodb::{results::InsertOneResult, Database, error::Error};

const COLLECTION_NAME: &str = "auditDetails";

pub async fn add(audit_detail: &AuditDetail, db: &Database) -> Result<InsertOneResult, Error> {
    crud_repository::add(audit_detail, COLLECTION_NAME, db).await
}
