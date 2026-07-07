use reqwest::Client;
use serde_json::Value;

#[derive(Clone)]
pub struct WhisperService {
    client: Client,
    base_url: String,
}

impl WhisperService {
    pub fn new(base_url: String, client: Client) -> Self {
        Self {
            client,
            base_url,
        }
    }

    pub async fn check_health(&self) -> Result<Value, reqwest::Error> {
        let endpoint = format!("{}/models", self.base_url);
        let response = self.client.get(&endpoint).send().await?;
        response.json::<Value>().await
    }

    pub async fn transcribe_audio(
        &self,
        file_name: String,
        content_type: String,
        data: bytes::Bytes
    ) -> Result<Value, reqwest::Error> {
        let file_part = reqwest::multipart::Part::stream(data)
            .file_name(file_name)
            .mime_str(&content_type)?;

        let form = reqwest::multipart::Form::new()
            .part("file", file_part)
            .text("model", "base")
            .text("response_format", "verbose_json");

        let endpoint = format!("{}/audio/transcriptions", self.base_url);
        let response = self.client.post(endpoint)
            .multipart(form)
            .send()
            .await?;

        response.json::<Value>().await
    }
}