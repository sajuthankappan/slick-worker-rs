use crate::lh_models::Report;
use crate::models::{
    AuditDetail, AuditProfile, AuditSummary, Categories, ConfigSettings, Performance, WebVitals,
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

    let mut config_settings = ConfigSettings::default();
    config_settings.set_throttling_method(lh_report.config_settings().throttling_method().clone());
    config_settings.set_throttling(lh_report.config_settings().throttling().clone());
    config_settings
        .set_emulated_form_factor(lh_report.config_settings().emulated_form_factor().clone());
    report.set_config_settings(config_settings);

    report
}

pub fn map_audit(
    page_name: String,
    report_id: ObjectId,
    device: String,
    lighthouse_version: String,
    url_audit_detail: &AuditDetail,
) -> AuditSummary {
    let summary = AuditSummary::new(
        page_name,
        AuditProfile::new(device, lighthouse_version),
        url_audit_detail.lighthouse_version().clone(),
        url_audit_detail.requested_url().clone(),
        url_audit_detail.final_url().clone(),
        url_audit_detail.fetch_time().clone(),
        url_audit_detail.categories().clone(),
        url_audit_detail.config_settings().clone(),
        url_audit_detail.web_vitals().clone(),
        report_id.clone(),
    );

    summary
}
