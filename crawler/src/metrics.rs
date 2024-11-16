use reqwest::Client;
use std::error::Error;
use serde_json::json;

pub struct MetricsClient {
    client: Client,
    base_url: String,
}

impl MetricsClient {
    pub fn new(base_url: String) -> Self {
        MetricsClient {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn increment(&self, metric: &str) -> Result<(), Box<dyn Error>> {
        self.client
            .post(&format!("{}/increment", self.base_url))
            .json(&json!({ "metric": metric }))
            .send()
            .await?;
        Ok(())
    }

    pub async fn set_gauge(&self, metric: &str, value: f64) -> Result<(), Box<dyn Error>> {
        self.client
            .post(&format!("{}/gauge", self.base_url))
            .json(&json!({ "metric": metric, "value": value }))
            .send()
            .await?;
        Ok(())
    }

    pub async fn observe_histogram(&self, metric: &str, value: f64) -> Result<(), Box<dyn Error>> {
        self.client
            .post(&format!("{}/histogram", self.base_url))
            .json(&json!({ "metric": metric, "value": value }))
            .send()
            .await?;
        Ok(())
    }
}
