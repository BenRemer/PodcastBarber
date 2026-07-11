use reqwest::{Client, StatusCode, multipart};
use serde_json::Value;
use testcontainers::{
    GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};
use tokio::fs;

mod common;

#[tokio::test]
async fn test_full_transcription_pipeline() {
    struct TestFile {
        name: &'static str,
        mime: &'static str,
        expected_phrase: &'static str,
    }

    let cases = vec![TestFile {
        name: "bologna-speech-english.wav",
        mime: "audio/wav",
        expected_phrase: "bologna",
    }];

    let whisper_image = GenericImage::new("fedirz/faster-whisper-server", "latest-cuda")
        .with_wait_for(WaitFor::message_on_stderr("Application startup complete"))
        .with_exposed_port(8000.tcp())
        // use tiny model for test
        .with_env_var("WHISPER__MODEL", "tiny");

    println!("Booting ephemeral Whisper container...");
    let container = whisper_image
        .start()
        .await
        .expect("Failed to start Whisper container");

    let host_port = container.get_host_port_ipv4(8000).await.unwrap();
    let dynamic_url = format!("http://127.0.0.1:{}/v1/audio/transcriptions", host_port);

    println!("Container ready! Bound to dynamic port: {}", host_port);

    let client = Client::new();

    // Iterate over the test cases
    for case in cases {
        println!("Testing file: {}", case.name);

        let audio_path = common::get_asset_path("test_audio.mp3");

        let audio_bytes = fs::read(&audio_path)
            .await
            .expect(&format!("Failed to read {}", case.name));

        // Construct the multipart form data using the struct fields
        let file_part = multipart::Part::bytes(audio_bytes)
            .file_name(case.name)
            .mime_str(case.mime)
            .unwrap();

        let form = multipart::Form::new().part("file", file_part);

        // Send the request to the ephemeral container
        let response = client
            .post(&dynamic_url)
            .multipart(form)
            .send()
            .await
            .expect("Failed to execute request against ephemeral container");

        // Assert the HTTP status code is exactly 200 OK
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Container API did not return 200 OK for file: {}",
            case.name
        );

        // Parse the JSON and assert the transcription is accurate
        let json: Value = response.json().await.expect("Response was not valid JSON");

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
        println!("transcript: {}", transcript);
    }
}
