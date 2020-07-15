mod audit;
mod data;
mod lh_client;
mod lh_data_mapper;

use audit::{audit_page, audit_profile, get_next_run_id};
use data::{
    repositories::{audit_detail_repository, site_repository},
    slick_db,
};
use env_logger;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, BasicQosOptions},
    types::FieldTable,
    Connection, ConnectionProperties,
};
use lh_client::LighthouseClient;
use log::info;
use serde::{Deserialize, Serialize};
use slick_models::ScoreParameters;

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

    let prefetch_count = 1;
    channel
        .basic_qos(prefetch_count, BasicQosOptions::default())
        .await
        .unwrap();

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
            info!("Receiving message");
            let data = std::str::from_utf8(&delivery.data).unwrap();
            let parameters = serde_json::from_str::<ScoreParameters>(&data).unwrap();

            if let Some(site_score_parameters) = parameters.site {
                let site_id = site_score_parameters.site_id;
                info!("Auditing site  {}..", &site_id);
                let site = site_repository::get_by_id(&site_id, &db)
                    .await
                    .unwrap()
                    .unwrap();
                let site_id = site.id().clone();
                let site_run_id = get_next_run_id(&site_id, &db).await;

                for page in site.pages() {
                    for profile in site.audit_profiles() {
                        let enabled = profile.enabled().unwrap_or(true);
                        if !enabled {
                            continue;
                        };

                        audit_profile(
                            &site_id,
                            &site_run_id,
                            &page,
                            &profile,
                            &site_score_parameters.cookie,
                            &lighthouse5_client,
                            &lighthouse6_client,
                            &db,
                        )
                        .await;
                    }
                }
            } else if let Some(page_score_parameters) = parameters.page {
                let page_audit_detail = audit_page(
                    page_score_parameters,
                    &lighthouse5_client,
                    &lighthouse6_client,
                )
                .await;
                let insert_result = audit_detail_repository::add(&page_audit_detail, &db)
                    .await
                    .unwrap();
                let audit_detail_id = insert_result.inserted_id.as_object_id().unwrap();
                info!("Inserted url audit detail {}", audit_detail_id);
            } else {
                panic!("No score parameters")
            }

            channel
                .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                .await
                .unwrap();
            info!("acknowledged message");
        }
    }
}
