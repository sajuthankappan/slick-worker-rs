use slick_models::SiteTread;
use wread_data_mongodb::crud_repository;
use wread_data_mongodb::mongodb::{error::Error, results::InsertOneResult, Database};

const COLLECTION_NAME: &str = "siteTreads";

pub async fn add(tread: &SiteTread, db: &Database) -> Result<InsertOneResult, Error> {
    crud_repository::add(tread, COLLECTION_NAME, db).await
}
