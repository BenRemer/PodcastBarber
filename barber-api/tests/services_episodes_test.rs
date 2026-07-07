use barber_api::models::episode::{Episode, EpisodeState};
use barber_api::models::podcast::Podcast;
use crate::common::TestContext;

mod common;

// todo refactor podcast to helper in ctx
#[tokio::test]
async fn test_save_episode() {
    let ctx = TestContext::setup().await;
    let feed_url = ctx.create_xml_feed_url("feed.xml").await;
    let audio_url = ctx.mock_audio_download("/test-audio.mp3").await;

    let metadata = ctx.rss_service.fetch_podcast_metadata(&feed_url).await.expect("Fetch failed");
    let podcast = Podcast::from(metadata);

    let podcast = ctx.podcast_service.subscribe_podcast(podcast).await.unwrap();

    let episode_list = ctx.rss_service
        .list_episodes(&feed_url, 1)
        .await
        .expect("Service failed to list episodes");
    assert_eq!(episode_list.len(), 1);

    let mut newest_episode = episode_list.into_iter().next().expect("Empty episode");
    newest_episode.audio_url = audio_url;
    let episode = newest_episode.clone().into_pending_episode(podcast.id);

    // Trigger the queue (Returns immediately as 'Pending')
    let saved_episode = ctx.episode_service
        .queue_episode_download(podcast, episode)
        .await
        .expect("Service failed to download episode");

    assert_eq!(saved_episode.title, newest_episode.title);

    // The Polling Loop: Wait for the background task to update the DB
    let mut attempts = 0;

    // Try 10 times, waiting 500ms between each check (5 seconds max)
    while attempts < 10 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let episode_state_opt: Option<String> = sqlx::query_scalar!(
            "SELECT state FROM episodes WHERE id = ?",
            saved_episode.id.to_string()
        )
        .fetch_optional(&ctx.pool) // Changed from fetch_one
        .await
        .expect("Database connection failed");

        // Check if the row exists yet
        if let Some(state) = episode_state_opt {
            if state == "Downloaded" || state == "Error" {
                break;
            }
        } else {
            // Row not found yet, just continue the loop and wait
            tracing::info!("Waiting for episode row to appear in DB...");
        };

        let final_episode = ctx
            .episode_service
            .get(saved_episode.id)
            .await
            .unwrap()
            .expect("Episode row with no existing episode");

        assert_eq!(final_episode.state, EpisodeState::Downloaded);

        // Break the loop if the state machine reached a terminal state
        if final_episode.state == EpisodeState::Downloaded || final_episode.state == EpisodeState::Error {
            break;
        }

        attempts += 1;
    }

    // todo cleanup
    let final_episode = ctx
        .episode_service
        .get(saved_episode.id)
        .await
        .unwrap()
        .expect("Episode row with no existing episode");

    // Assert the final outcome of the background task
    assert_eq!(final_episode.state, EpisodeState::Downloaded, "Background task failed or timed out");
    assert!(final_episode.local_file_path.is_some(), "File path should be populated");
}

#[tokio::test]
async fn test_save_nonexistent_episode() {
    let ctx = TestContext::setup().await;

    let podcast = Podcast {
        id: Default::default(),
        title: "".to_string(),
        feed_url: "".to_string(),
        image_url: None,
        description: None,
        author: None,
    };

    let episode = Episode {
        id: Default::default(),
        podcast_id: Default::default(),
        guid: "".to_string(),
        title: "".to_string(),
        audio_url: "".to_string(),
        local_file_path: None,
        state: EpisodeState::Pending,
    };

    let failure = ctx.episode_service
        .queue_episode_download(podcast, episode)
        .await;

    assert!(failure.is_err());
}