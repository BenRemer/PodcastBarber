use barber_api::models::episode::{Episode, EpisodeState};
use crate::common::TestContext;
use crate::common::builder::PodcastFixtureBuilder;

mod common;

#[tokio::test]
async fn test_queue_episode_download() {
    let ctx = TestContext::setup().await;
    let (podcast, mut episodes) = PodcastFixtureBuilder::new(&ctx)
        .subscribed()
        .with_episodes(1)
        .build()
        .await;
    let episode = episodes.pop().expect("no episodes");

    // Trigger the queue (Returns immediately as 'Pending')
    let saved_episode = ctx.episode_service
        .queue_episode_download(podcast, episode.clone())
        .await
        .expect("Service failed to download episode");

    assert_eq!(saved_episode.title, episode.title);

    // The Polling Loop: Wait for the background task to update the DB
    let mut attempts = 0;
    // Try 10 times, waiting 500ms between each check (5 seconds max)
    while attempts < 10 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let episode_state_opt: Option<String> = sqlx::query_scalar!(
            "SELECT state FROM episodes WHERE id = ?",
            saved_episode.id.to_string()
        )
        .fetch_optional(&ctx.pool)
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
        attempts += 1;
    }

    let final_episode = ctx
        .episode_service
        .get(&saved_episode.id)
        .await
        .unwrap()
        .expect("Episode row with no existing episode");

    // Assert the final outcome of the background task
    assert_eq!(final_episode.state, EpisodeState::Downloaded, "Background task failed or timed out");
    assert_eq!(final_episode.title, episode.title);
    assert!(final_episode.local_file_path.is_some(), "File path should be populated");
}

#[tokio::test]
async fn test_save_nonexistent_episode() {
    let ctx = TestContext::setup().await;
    let podcast = ctx.create_subscribed_podcast(None, None).await;

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

#[tokio::test]
async fn test_get_episode() {
    let ctx = TestContext::setup().await;
    let (_, mut episodes) = PodcastFixtureBuilder::new(&ctx)
        .subscribed()
        .with_episodes(2)
        .build()
        .await;

    let episode = episodes.pop().expect("no episodes");

    let saved_episode = ctx.episode_service
        .get(&episode.id)
        .await
        .expect("Database connection failed")
        .expect("Episode row with no existing episode");

    assert_eq!(saved_episode, episode);
}

#[tokio::test]
async fn test_get_all_episodes() {
    let ctx = TestContext::setup().await;
    let (podcast, _) = PodcastFixtureBuilder::new(&ctx)
        .subscribed()
        .with_episodes(3)
        .build()
        .await;

    // Run your service assertion!
    let fetched = ctx.episode_service.get_episodes_by_podcast(&podcast.id).await.unwrap();
    assert_eq!(fetched.len(), 3);
}

#[tokio::test]
async fn test_delete_episode() {
    let ctx = TestContext::setup().await;
    let (podcast, episodes) = PodcastFixtureBuilder::new(&ctx)
        .subscribed()
        .with_episodes(3)
        .build()
        .await;

    let episode = episodes.into_iter().nth(2).expect("no episode");

    let saved_episode = ctx.episode_service
        .queue_episode_download(podcast, episode.clone())
        .await
        .expect("Service failed to download episode");

    assert_eq!(saved_episode.title, episode.title);

    for _ in 0..10 {
        // wait for item to download
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    let final_episode = ctx
        .episode_service
        .get(&saved_episode.id)
        .await
        .unwrap()
        .expect("Episode row with no existing episode");

    assert_eq!(final_episode.state, EpisodeState::Downloaded, "Background task failed or timed out");

    let result = ctx.episode_service.delete_episode(final_episode).await;

    assert!(!result.is_err());
}