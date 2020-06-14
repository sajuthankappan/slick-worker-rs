use crate::models::PageScoreReport;
use wread_data_mongodb::crud_repository;
use wread_data_mongodb::mongodb::{results::InsertOneResult, Database, error::Error};

const COLLECTION_NAME: &str = "reports";

pub async fn add(report: &PageScoreReport, db: &Database) -> Result<InsertOneResult, Error> {
    crud_repository::add(report, COLLECTION_NAME, db).await
}
