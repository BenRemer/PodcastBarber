use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

pub async fn create(server: &wiremock::MockServer) -> String {
    let audio_path = format!("/audio-{}.mp3", Uuid::new_v4());

    Mock::given(method("GET"))
        .and(path(audio_path.clone()))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0u8; 1024]))
        .mount(server)
        .await;

    format!("{}{}", server.uri(), audio_path)
}
