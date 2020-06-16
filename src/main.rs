use env_logger;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions},
    types::FieldTable,
    Connection, ConnectionProperties,
};
use log::info;
use serde::{Deserialize, Serialize};

mod lh_client;
use lh_client::LighthouseClient;

mod lh_models;

mod models;
use models::PageScoreParameters;

mod data;
mod lh_data_mapper;
use data::{repositories::report_repository, slick_db};

#[derive(Deserialize, Serialize, Debug)]
struct WorkerConfig {
    amqp_uri: String,
    queue_name: String,
    lighthouse6_api_url: String,
    lighthouse5_api_url: String,
    db_uri: String,
    db_name: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting Slick Worker..");

    let mut raw_config = config::Config::default();
    raw_config
        .merge(config::Environment::with_prefix("SLICK"))
        .unwrap();
    let worker_config = raw_config.try_into::<WorkerConfig>().unwrap();

    let db = slick_db::get_db(worker_config.db_uri.clone(), worker_config.db_name.clone()).await;
    info!("Connected to db");

    let amqp_addr = worker_config.amqp_uri;
    let lighthouse6_api_url = worker_config.lighthouse6_api_url.clone();
    let lighthouse5_api_url = worker_config.lighthouse5_api_url.clone();

    let conn = Connection::connect(
        &amqp_addr,
        ConnectionProperties::default().with_default_executor(8),
    )
    .await
    .unwrap();
    let channel = conn.create_channel().await.unwrap();
    info!("Connected to amqp");

    let consumer = channel
        .basic_consume(
            "score-requests",
            "slick-worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    let lighthouse6_client = LighthouseClient::new(&lighthouse6_api_url);
    let lighthouse5_client = LighthouseClient::new(&lighthouse5_api_url);

    info!("Waiting for messages");

    for delivery in consumer {
        if let Ok((_channel, delivery)) = delivery {
            info!("receiving message");
            let data = std::str::from_utf8(&delivery.data).unwrap();
            let parameters: PageScoreParameters = serde_json::from_str(&data).unwrap();
            let lh_results = if let Some(lh_version) = &parameters.lighthouse_version {
                if lh_version.clone() == String::from("5") {
                    lighthouse5_client.generate_report(parameters).await
                } else {
                    lighthouse6_client.generate_report(parameters).await
                }
            } else {
                lighthouse6_client.generate_report(parameters).await
            };
            let lh_data = lh_results
                .results()
                .get(lh_results.best_score_index().to_owned())
                .unwrap();
            let results = lh_data_mapper::map_lh_data(lh_data);
            let insert_result = report_repository::add(&results, &db).await.unwrap();
            let report_id = insert_result.inserted_id.as_object_id().unwrap();
            println!("Inserted report {}", report_id);
            channel
                .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                .await
                .unwrap();
            info!("acknowledged message");
        }
    }
}
