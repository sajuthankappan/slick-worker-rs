use slick_models::AuditSummary;
use wread_data_mongodb::crud_repository;
use wread_data_mongodb::mongodb::{error::Error, results::InsertOneResult, Database};

const COLLECTION_NAME: &str = "auditSummaries";

pub async fn add(audit_summary: &AuditSummary, db: &Database) -> Result<InsertOneResult, Error> {
    crud_repository::add(audit_summary, COLLECTION_NAME, db).await
}
