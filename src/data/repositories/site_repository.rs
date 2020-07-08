use slick_models::Site;
use wread_data_mongodb::crud_repository;
use wread_data_mongodb::mongodb::bson::{doc, oid::ObjectId};
use wread_data_mongodb::mongodb::{
    error::Error,
    options::{FindOneAndUpdateOptions, ReturnDocument},
    Database,
};

const COLLECTION_NAME: &str = "sites";

pub async fn get_by_id(id: &String, db: &Database) -> Result<Option<Site>, Error> {
    let object_id = ObjectId::with_string(id.as_str()).unwrap();
    crud_repository::find_by_id(&object_id, COLLECTION_NAME, &db).await
}

pub async fn increment_last_run_id(id: &ObjectId, db: &Database) -> Result<Option<Site>, Error> {
    let filter = doc! { "_id": id };
    let update = doc! { "$inc": { "lastRunId": 1 } };
    let builder = FindOneAndUpdateOptions::builder();
    let options = Some(builder.return_document(ReturnDocument::After).build());
    crud_repository::find_one_and_update(filter, update, options, COLLECTION_NAME, &db).await
}
