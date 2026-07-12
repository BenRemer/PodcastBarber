use crate::error::AppError;
use reqwest::Client;
use serde_json::Value;

#[derive(Clone)]
pub struct TranscribeCore {
    pub(crate) base_url: String,
    pub(crate) client: Client,
}

impl TranscribeCore {
    pub fn new(base_url: String, client: Client) -> Self {
        Self { base_url, client }
    }

    pub async fn check_health(&self) -> Result<Value, AppError> {
        let endpoint = format!("{}/models", self.base_url);
        let response = self.client.get(&endpoint).send().await?;
        let json = response.json::<Value>().await?;
        Ok(json)
    }

    /// Send data to whisper and return json transcript
    pub async fn transcribe_audio(
        &self,
        file_name: String,
        content_type: String,
        data: bytes::Bytes,
    ) -> Result<Value, AppError> {
        let file_part = reqwest::multipart::Part::stream(data)
            .file_name(file_name)
            .mime_str(&content_type)?;

        let form = reqwest::multipart::Form::new()
            .part("file", file_part)
            .text("model", "base")
            .text("response_format", "verbose_json");

        // let endpoint = format!("{}/episode/transcriptions", self.base_url);
        let endpoint = format!("{}/audio/transcriptions", self.base_url);
        let response = self.client.post(endpoint).multipart(form).send().await?;

        let transcript = response.json::<Value>().await?;
        Ok(transcript)
    }
}
