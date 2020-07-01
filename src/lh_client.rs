use slick_models::lh_models::AllAttemptReports;
use slick_models::PageScoreParameters;
use log::info;
use reqwest::{Client, StatusCode, Url};

pub struct LighthouseClient {
    report_url: reqwest::Url,
}

impl LighthouseClient {
    pub fn new(api_url: &String) -> LighthouseClient {
        let report_url = format!("{}/report", api_url);
        LighthouseClient {
            report_url: Url::parse(&report_url).unwrap(),
        }
    }
    pub async fn generate_report(&self, parameters: PageScoreParameters) -> AllAttemptReports {
        info!(
            "auditing {} with lighthouse {} on {}",
            &parameters.url,
            &parameters
                .lighthouse_version
                .clone()
                .unwrap_or(String::from("6")),
            &parameters.device.clone().unwrap_or(String::from("mobile"))
        );
        let client = Client::new();
        let res = client
            .post(self.report_url.as_str())
            .json(&parameters)
            .send()
            .await
            .unwrap();
        if res.status().clone() != StatusCode::OK {
            todo!("Implement error handling")
        }

        let results = res.json::<AllAttemptReports>().await.unwrap();
        results
    }
}
