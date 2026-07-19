use crate::common;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

pub async fn create(server: &wiremock::MockServer, audio: Option<&str>) -> String {
    let audio_path = format!("/audio-{}.wav", Uuid::new_v4());
    let audio = audio.unwrap_or("bologna-speech-english.wav");

    let asset_path = common::get_asset_path(audio);

    let audio_bytes = tokio::fs::read(asset_path)
        .await
        .expect("failed reading audio fixture");

    Mock::given(method("GET"))
        .and(path(audio_path.clone()))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "audio/wav")
                .set_body_bytes(audio_bytes),
        )
        .mount(server)
        .await;

    format!("{}{}", server.uri(), audio_path)
}
