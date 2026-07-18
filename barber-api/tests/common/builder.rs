use crate::common::context::TestContext;
use barber_api::models::episode::Episode;
use barber_api::models::podcast::Podcast;

pub struct PodcastFixtureBuilder<'a> {
    ctx: &'a TestContext,
    title: Option<String>,
    feed_url: Option<String>,
    auto_subscribe: bool,
    with_episodes: usize,
}

impl<'a> PodcastFixtureBuilder<'a> {
    pub fn new(ctx: &'a TestContext) -> Self {
        Self {
            ctx,
            title: None,
            feed_url: None,
            auto_subscribe: false,
            with_episodes: 0,
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn subscribed(mut self) -> Self {
        self.auto_subscribe = true;
        self
    }

    pub fn with_episodes(mut self, count: usize) -> Self {
        self.with_episodes = count;
        self
    }

    pub async fn build(self) -> (Podcast, Vec<Episode>) {
        let mut podcast = self
            .ctx
            .create_podcast(self.title.as_deref(), self.feed_url.as_deref())
            .await;

        if self.auto_subscribe || self.with_episodes > 0 {
            podcast = self
                .ctx
                .podcast_service
                .subscribe_podcast(podcast)
                .await
                .unwrap();
        }

        let mut episodes = Vec::new();
        for _ in 0..self.with_episodes {
            episodes.push(
                self.ctx
                    .create_test_episode(Some(podcast.clone()), None, None)
                    .await,
            );
        }

        (podcast, episodes)
    }
}
