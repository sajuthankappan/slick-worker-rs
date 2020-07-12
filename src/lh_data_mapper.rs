use slick_models::lh_models::Report;
use slick_models::{
    AuditDetail, AuditProfile, AuditSummary, Categories, ConfigSettings, Performance,
    WebVitals,
};
use wread_data_mongodb::mongodb::bson::oid::ObjectId;

pub fn map_lh_data(lh_report: &Report) -> AuditDetail {
    let mut report = AuditDetail::default();
    report.set_lighthouse_version(lh_report.lighthouse_version().clone());
    report.set_requested_url(lh_report.requested_url().clone());
    report.set_final_url(lh_report.final_url().clone());
    report.set_fetch_time(lh_report.fetch_time().clone());

    let mut performance = Performance::default();
    performance.set_id(lh_report.categories().performance().id().clone());
    performance.set_title(lh_report.categories().performance().title().clone());
    performance.set_score(lh_report.categories().performance().score().clone());

    let mut categories = Categories::default();
    categories.set_performance(performance);
    report.set_categories(categories);

    let mut web_vitals = WebVitals::default();
    web_vitals.set_first_contentful_paint(lh_report.audits().first_contentful_paint().clone());
    web_vitals.set_speed_index(lh_report.audits().speed_index().clone());
    web_vitals.set_largest_contentful_paint(lh_report.audits().largest_contentful_paint().clone());
    web_vitals.set_interactive(lh_report.audits().interactive().clone());
    web_vitals.set_total_blocking_time(lh_report.audits().total_blocking_time().clone());
    web_vitals.set_cumulative_layout_shift(lh_report.audits().cumulative_layout_shift().clone());
    web_vitals.set_first_meaningful_paint(lh_report.audits().first_meaningful_paint().clone());
    web_vitals.set_max_potential_fid(lh_report.audits().max_potential_fid().clone());
    web_vitals.set_first_cpu_idle(lh_report.audits().first_cpu_idle().clone());
    report.set_web_vitals(web_vitals);

    report.set_largest_contentful_paint_element(
        lh_report
            .audits()
            .largest_contentful_paint_element()
            .clone(),
    );

    report.set_network_requests(Some(lh_report.audits().network_requests().clone()));
    report.set_resource_summary(Some(lh_report.audits().resource_summary().clone()));
    report.set_third_party_summary(Some(lh_report.audits().third_party_summary().clone()));
    report.set_screenshot_thumbnails(Some(lh_report.audits().screenshot_thumbnails().clone()));
    report.set_uses_responsive_images(Some(lh_report.audits().uses_responsive_images().clone()));
    report.set_uses_optimized_images(Some(lh_report.audits().uses_optimized_images().clone()));
    report.set_uses_webp_images(Some(lh_report.audits().uses_webp_images().clone()));
    report.set_offscreen_images(Some(lh_report.audits().offscreen_images().clone()));
    report.set_uses_http2(lh_report.audits().uses_http2().clone());

    report.set_main_thread_tasks(Some(lh_report.audits().main_thread_tasks().clone()));
    report.set_network_rtt(Some(lh_report.audits().network_rtt().clone()));
    report.set_bootup_time(lh_report.audits().bootup_time().clone());
    report.set_main_thread_work_breakdown(lh_report.audits().main_thread_work_breakdown().clone());
    report.set_uses_rel_preconnect(lh_report.audits().uses_rel_preconnect().clone());
    report.set_network_server_latency(lh_report.audits().network_server_latency().clone());

    let mut config_settings = ConfigSettings::default();
    config_settings.set_throttling_method(lh_report.config_settings().throttling_method().clone());
    config_settings.set_throttling(lh_report.config_settings().throttling().clone());
    config_settings
        .set_emulated_form_factor(lh_report.config_settings().emulated_form_factor().clone());
    report.set_config_settings(config_settings);

    report
}

pub fn map_audit(
    site_id: ObjectId,
    site_run_id: i32,
    page_id: String,
    audit_detail_id: ObjectId,
    audit_profile: AuditProfile,
    audit_detail: &AuditDetail,
) -> AuditSummary {
    let summary = AuditSummary::new(
        site_id,
        site_run_id,
        page_id,
        audit_profile.id().clone(),
        audit_profile.clone(),
        audit_detail.fetch_time().clone(),
        audit_detail.categories().clone(),
        audit_detail.config_settings().clone(),
        audit_detail.web_vitals().clone(),
        audit_detail_id.clone(),
    );

    summary
}
