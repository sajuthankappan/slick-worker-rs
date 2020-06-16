use crate::lh_models::PageScore;
use crate::models::{Categories, KeyAudits, PageScoreReport, Performance, ConfigSettings};

pub fn map_lh_data(lh_score: &PageScore) -> PageScoreReport {
	let mut report = PageScoreReport::default();
	report.set_lighthouse_version(lh_score.lighthouse_version().clone());
	report.set_requested_url(lh_score.requested_url().clone());
	report.set_final_url(lh_score.final_url().clone());
	report.set_fetch_time(lh_score.fetch_time().clone());

	let mut performance = Performance::default();
	performance.set_id(lh_score.categories().performance().id().clone());
	performance.set_title(lh_score.categories().performance().title().clone());
	performance.set_score(lh_score.categories().performance().score().clone());

	let mut categories = Categories::default();
	categories.set_performance(performance);
	report.set_categories(categories);

	let mut key_audits = KeyAudits::default();
	key_audits.set_first_contentful_paint(lh_score.audits().first_contentful_paint().clone());
	key_audits.set_speed_index(lh_score.audits().speed_index().clone());
	key_audits.set_largest_contentful_paint(lh_score.audits().largest_contentful_paint().clone());
	key_audits.set_interactive(lh_score.audits().interactive().clone());
	key_audits.set_total_blocking_time(lh_score.audits().total_blocking_time().clone());
	key_audits.set_cumulative_layout_shift(lh_score.audits().cumulative_layout_shift().clone());
	key_audits.set_first_meaningful_paint(lh_score.audits().first_meaningful_paint().clone());
	key_audits.set_max_potential_fid(lh_score.audits().max_potential_fid().clone());
	key_audits.set_first_cpu_idle(lh_score.audits().first_cpu_idle().clone());
	report.set_key_audits(key_audits);

	let mut config_settings = ConfigSettings::default();
	config_settings.set_throttling_method(lh_score.config_settings().throttling_method().clone());
	config_settings.set_throttling(lh_score.config_settings().throttling().clone());
	config_settings.set_emulated_form_factor(lh_score.config_settings().emulated_form_factor().clone());

	report
}
