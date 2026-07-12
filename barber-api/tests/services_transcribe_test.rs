use crate::common::TestContext;
use barber_api::services::transcribe::core::TranscribeCore;
use barber_api::utils::get_content_type;
use reqwest::Client;
use tokio::fs;

mod common;

#[tokio::test]
async fn test_core_transcribe() {
    struct TestFile {
        name: &'static str,
        mime: &'static str,
        expected_phrase: &'static str,
    }

    let cases = vec![TestFile {
        name: "bologna-speech-english.wav",
        mime: "episode/wav",
        expected_phrase: "hitchcock",
    }];

    let ctx = TestContext::setup().await;
    let (_whisper_container, dynamic_url) = ctx.start_whisper_sidecar().await;

    let client = Client::new();
    let core = TranscribeCore::new(dynamic_url, client);

    // Iterate over the test cases
    for case in cases {
        println!("Testing file: {}", case.name);

        let audio_path = common::get_asset_path(case.name);
        let audio_bytes = fs::read(&audio_path)
            .await
            .expect(&format!("Failed to read {}", case.name));
        let content_type = get_content_type(&audio_bytes);

        let json = core
            .transcribe_audio(case.name.to_string(), content_type, audio_bytes.into())
            .await
            .expect("expected transcription");

        let transcript = json
            .get("text")
            .expect("JSON payload missing 'text' field")
            .as_str()
            .expect("'text' field is not a string");

        // Assert using the struct's dynamic phrase
        assert!(
            transcript.to_lowercase().contains(case.expected_phrase),
            "Whisper failed to transcribe the expected phrase for {}. Got: {}",
            case.name,
            transcript
        );
    }
}
