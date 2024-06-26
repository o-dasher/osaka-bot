use sqlx::{Pool, Postgres};
use std::sync::Arc;
use submit::ScoreSubmitter;

pub mod beatmap_cache;
pub mod submit;

pub struct Manager {
    pub beatmap_cache_manager: Arc<beatmap_cache::Manager>,
    pub submit_manager: ScoreSubmitter,
}

impl Manager {
    #[must_use]
    pub fn new(pool: Pool<Postgres>, rosu: Arc<rosu_v2::Osu>) -> Self {
        let beatmap_cache = Arc::new(beatmap_cache::Manager::new());

        Self {
            beatmap_cache_manager: beatmap_cache.clone(),
            submit_manager: ScoreSubmitter::new(beatmap_cache, pool, rosu),
        }
    }
}
