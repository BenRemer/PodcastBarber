mod common;

use crate::common::builder::PodcastFixtureBuilder;
use crate::common::mocks;
use barber_api::models::episode::EpisodeState;
use barber_api::processors::coordinator::PipelineEvent;
use barber_api::storage::repository::detection::DetectionStore;
use barber_api::storage::repository::transcript::TranscriptStore;
use common::context::TestContext;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_full_audio_pipeline() {
    let (event_tx, mut event_rx) = mpsc::channel::<PipelineEvent>(100);
    let (_container, whisper_url) = mocks::whisper::start_sidecar().await;
    let ctx = TestContext::builder()
        .with_workers()
        .whisper_url(whisper_url)
        .with_pipeline_events(event_tx)
        .build()
        .await;
    let (podcast, mut episodes) = PodcastFixtureBuilder::new(&ctx)
        .subscribed()
        .audio("svs.mp3")
        .episodes(1)
        .build()
        .await;
    let episode = episodes.pop().expect("no episodes");

    // Queue download
    let _ = ctx
        .episode_service
        .queue_episode_download(podcast, episode.clone())
        .await
        .expect("Service failed to download episode");

    while let Some(event) = event_rx.recv().await {
        match event {
            PipelineEvent::DownloadComplete(id) if id == episode.id => {
                let episode = ctx
                    .episode_repository
                    .get(&id)
                    .await
                    .unwrap()
                    .expect("no episode");
                assert_eq!(EpisodeState::Downloaded, episode.state);
            }
            PipelineEvent::TranscriptionComplete(id, error) if id == episode.id => {
                if let Some(error) = error {
                    panic!("Failure {:?}", error);
                }
                let episode = ctx
                    .episode_repository
                    .get(&id)
                    .await
                    .unwrap()
                    .expect("no episode");
                assert_eq!(EpisodeState::Transcribed, episode.state);
            }
            PipelineEvent::DetectionComplete(id, error) if id == episode.id => {
                if let Some(error) = error {
                    panic!("Failure {:?}", error);
                }
                let episode = ctx
                    .episode_repository
                    .get(&id)
                    .await
                    .unwrap()
                    .expect("no episode");
                assert_eq!(EpisodeState::Detected, episode.state);

                break;
            }
            _ => {}
        }
    }
    let _ = ctx
        .transcript_repository
        .get_by_episode_id(&episode.id)
        .await
        .unwrap()
        .expect("transcript missing");

    let detection = ctx
        .detection_repository
        .get_detection_by_episode(&episode.id)
        .await
        .unwrap()
        .expect("detection missing");

    for segment in &detection.segments {
        if segment.is_ad {
            println!("{:#?}", segment);
        }
    }

    assert!(!detection.segments.is_empty());

    let segments = detection.segments;
    assert!(segments.len() > 4);
}
