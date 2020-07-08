use env_logger;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, BasicQosOptions},
    types::FieldTable,
    Connection, ConnectionProperties,
};
use log::info;
use serde::{Deserialize, Serialize};

mod lh_client;
use lh_client::LighthouseClient;

use slick_models::{AuditDetail, PageScoreParameters, ScoreParameters};

mod data;
mod lh_data_mapper;
use data::{
    repositories::{audit_detail_repository, audit_summary_repository, site_repository},
    slick_db,
};

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
            info!("receiving message");
            let data = std::str::from_utf8(&delivery.data).unwrap();
            let parameters = serde_json::from_str::<ScoreParameters>(&data).unwrap();

            if let Some(site_score_parameters) = parameters.site {
                let site_id = site_score_parameters.site_id;
                let site = site_repository::get_by_id(&site_id, &db)
                    .await
                    .unwrap()
                    .unwrap();
                let site_id = site.id().clone();

                let site_run_id = site_repository::increment_last_run_id(&site_id, &db)
                    .await
                    .unwrap()
                    .unwrap()
                    .last_run_id()
                    .clone();

                for page in site.pages() {
                    for profile in site.audit_profiles() {
                        let page_score_parameters = PageScoreParameters {
                            url: page.url().clone(),
                            device: Some(profile.device().clone()),
                            throttling: None,
                            attempts: None,
                            lighthouse_version: Some(profile.lighthouse_version().clone()),
                        };
                        let audit_detail = audit_page(
                            page_score_parameters,
                            &lighthouse5_client,
                            &lighthouse6_client,
                        )
                        .await;

                        let detail_insert_result = audit_detail_repository::add(&audit_detail, &db)
                            .await
                            .unwrap();
                        let audit_detail_id =
                            detail_insert_result.inserted_id.as_object_id().unwrap();
                        info!("Inserted audit detail {}", &audit_detail_id);

                        let audit_summary = lh_data_mapper::map_audit(
                            site_id.clone(),
                            site_run_id.clone(),
                            page.id().clone(),
                            audit_detail_id.clone(),
                            profile.clone(),
                            &audit_detail,
                        );
                        let summary_insert_result =
                            audit_summary_repository::add(&audit_summary, &db)
                                .await
                                .unwrap();
                        let audit_summary_id =
                            summary_insert_result.inserted_id.as_object_id().unwrap();
                        info!("Inserted audit summary {}", &audit_summary_id);
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

async fn audit_page(
    page_score_parameters: PageScoreParameters,
    lighthouse5_client: &LighthouseClient,
    lighthouse6_client: &LighthouseClient,
) -> AuditDetail {
    let lh_all_attempt_reports = if let Some(lh_version) = &page_score_parameters.lighthouse_version
    {
        if lh_version.clone() == String::from("5") {
            lighthouse5_client
                .generate_report(page_score_parameters)
                .await
        } else {
            lighthouse6_client
                .generate_report(page_score_parameters)
                .await
        }
    } else {
        lighthouse6_client
            .generate_report(page_score_parameters)
            .await
    };
    let lh_report = lh_all_attempt_reports
        .reports()
        .get(lh_all_attempt_reports.best_score_index().to_owned())
        .unwrap();
    let detail = lh_data_mapper::map_lh_data(lh_report);
    detail
}
