use crate::data::repositories::{
    audit_detail_repository, audit_summary_repository, site_repository,
};
use crate::lh_client::LighthouseClient;
use crate::lh_data_mapper;
use log::info;
use slick_models::{AuditDetail, AuditProfile, Page, PageScoreParameters};
use wread_data_mongodb::mongodb::{bson::oid::ObjectId, Database};

pub async fn audit_profile(
    site_id: &ObjectId,
    site_run_id: &i32,
    page: &Page,
    profile: &AuditProfile,
    lighthouse5_client: &LighthouseClient,
    lighthouse6_client: &LighthouseClient,
    db: &Database,
) {
    let page_score_parameters = PageScoreParameters {
        url: page.url().clone(),
        device: Some(profile.device().clone()),
        throttling: None,
        attempts: None,
        lighthouse_version: Some(profile.lighthouse_version().clone()),
        blocked_url_patterns: profile.blocked_url_patterns().clone(),
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
    let audit_detail_id = detail_insert_result.inserted_id.as_object_id().unwrap();
    info!("Inserted audit detail {}", &audit_detail_id);

    let audit_summary = lh_data_mapper::map_audit(
        site_id.clone(),
        site_run_id.clone(),
        page.id().clone(),
        audit_detail_id.clone(),
        profile.clone(),
        &audit_detail,
    );
    let summary_insert_result = audit_summary_repository::add(&audit_summary, &db)
        .await
        .unwrap();
    let audit_summary_id = summary_insert_result.inserted_id.as_object_id().unwrap();
    info!("Inserted audit summary {}", &audit_summary_id);
}

pub async fn audit_page(
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

pub async fn get_next_run_id(site_id: &ObjectId, db: &Database) -> i32 {
    let site_run_id = site_repository::increment_last_run_id(site_id, db)
        .await
        .unwrap()
        .unwrap()
        .last_run_id()
        .clone();
    site_run_id
}
