use itertools::Itertools;
use rosu_pp::any::PerformanceAttributes;
use sqlx::{types::BigDecimal, Postgres, QueryBuilder};
use std::{collections::HashSet, sync::Arc};

use rosu_v2::model::{score::Score, GameMode};
use strum::Display;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    RwLock,
};

use crate::{
    managers::osu::OsuManager,
    utils::id_locked::{IDLocker, IDLockerError},
    OsakaData, OsakaManagers,
};

use super::beatmap::BeatmapCacheError;

#[derive(derive_more::From)]
pub enum SubmissionID {
    ByStoredID(u32),
    ByUsername(String),
}

#[derive(Display)]
#[strum(serialize_all = "lowercase")]
pub enum SubmittableMode {
    Osu,
    Taiko,
    Mania,
}

impl TryFrom<GameMode> for SubmittableMode {
    type Error = SubmissionError;

    fn try_from(value: GameMode) -> Result<Self, Self::Error> {
        match value {
            GameMode::Osu => Ok(Self::Osu),
            GameMode::Taiko => Ok(Self::Taiko),
            GameMode::Mania => Ok(Self::Mania),
            GameMode::Catch => Err(SubmissionError::UnsupportedMode),
        }
    }
}

impl From<SubmittableMode> for GameMode {
    fn from(val: SubmittableMode) -> Self {
        match val {
            SubmittableMode::Osu => Self::Osu,
            SubmittableMode::Taiko => Self::Taiko,
            SubmittableMode::Mania => Self::Mania,
        }
    }
}

pub struct ScoreSubmitter {
    locker: IDLocker<String>,
}

pub struct ReadyScoreSubmitter {
    submitter: Arc<RwLock<ScoreSubmitter>>,
    sender: Sender<(usize, usize)>,
}

impl Default for ScoreSubmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(thiserror::Error, Debug, derive_more::From)]
pub enum SubmissionError {
    #[error("This command does not support this mode.")]
    UnsupportedMode,

    #[error("Missing dependencies.")]
    MissingDependencies,

    #[error(transparent)]
    Sqlx(sqlx::Error),

    #[error(transparent)]
    IdLocker(IDLockerError),

    #[error(transparent)]
    RosuV2(rosu_v2::error::OsuError),

    #[error(transparent)]
    FetchBeatmap(BeatmapCacheError),

    #[error(transparent)]
    Io(std::io::Error),
}

impl ScoreSubmitter {
    pub fn new() -> Self {
        Self {
            locker: IDLocker::new(),
        }
    }

    pub fn begin_submission(
        submitter: &Arc<RwLock<ScoreSubmitter>>,
    ) -> (ReadyScoreSubmitter, Receiver<(usize, usize)>) {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);

        (
            ReadyScoreSubmitter {
                submitter: submitter.clone(),
                sender,
            },
            receiver,
        )
    }
}

impl ReadyScoreSubmitter {
    pub async fn submit_scores(
        &self,
        osu_id: impl Into<SubmissionID>,
        mode: GameMode,
        data: OsakaData,
    ) -> Result<(), SubmissionError> {
        let submit_mode = SubmittableMode::try_from(mode)?;
        let submitter = self.submitter.read().await;

        let OsakaData {
            pool,
            rosu,
            managers,
            ..
        } = data;

        let OsakaManagers { osu_manager, .. } = managers;
        let OsuManager {
            beatmap_cache_manager,
            ..
        } = osu_manager;

        // This cast should be safe
        let mode_bits = mode as i16;

        let osu_id = match osu_id.into() {
            SubmissionID::ByStoredID(id) => id,
            SubmissionID::ByUsername(username) => rosu.user(username).await?.user_id,
        };

        let locker_guard = submitter.locker.lock(osu_id.to_string()).await?;

        let osu_scores = rosu.user_scores(osu_id).limit(100).mode(mode).await?;

        #[derive(sqlx::FromRow)]
        struct ExistingScore {
            id: BigDecimal,
        }

        let rika_osu_scores: Vec<ExistingScore> = sqlx::query_as(&format!(
            "
			SELECT s.score_id FROM osu_score s
			JOIN {submit_mode}_performance pp ON s.id = pp.score_id
			WHERE s.osu_user_id = ?
			"
        ))
        .bind(i64::from(osu_id))
        .fetch_all(&pool)
        .await?;

        let existing_scores: HashSet<_> = rika_osu_scores.into_iter().map(|s| s.id).collect();

        let new_scores = osu_scores
            .iter()
            .filter_map(|s| {
                s.score_id.and_then(|score_id| {
                    let is_new = !existing_scores.contains(&score_id.into());

                    is_new.then_some((score_id, s))
                })
            })
            .collect_vec();

        if new_scores.is_empty() {
            return Ok(());
        }

        let mut performance_information: Vec<(PerformanceAttributes, (&Score, &u64))> = vec![];

        for (i, (score_id, score)) in new_scores.iter().enumerate() {
            let ss = &score.statistics;
            performance_information.push((
                rosu_pp::Performance::new(
                    rosu_pp::Difficulty::new()
                        .mods(score.mods.bits())
                        .calculate(&rosu_pp::Beatmap::from_bytes(
                            &beatmap_cache_manager.get_beatmap_file(score.map_id).await?,
                        )?),
                )
                .n300(ss.count_300)
                .n100(ss.count_100)
                .n50(ss.count_50)
                .n_geki(ss.count_geki)
                .n_katu(ss.count_katu)
                .misses(ss.count_miss)
                .calculate(),
                (*score, score_id),
            ));
            let _ = self.sender.send((i + 1, new_scores.len())).await;
        }

        let mut tx = pool.begin().await?;

        QueryBuilder::<Postgres>::new(
            "
			INSERT INTO osu_score (score_id, osu_user_id, map_id, mods, mode)
			",
        )
        .push_values(
            &performance_information,
            |mut b, (.., (score, score_id))| {
                b.push_bind(BigDecimal::from(**score_id))
                    .push_bind(i64::from(osu_id))
                    .push_bind(i64::from(score.map_id))
                    .push_bind(i64::from(score.mods.bits()))
                    .push_bind(mode_bits);
            },
        );

        QueryBuilder::<Postgres>::new(format!(
            "INSERT INTO {submit_mode}_performance (score_id, overall, {})",
            match submit_mode {
                SubmittableMode::Osu => "aim, speed, flashlight, accuracy",
                SubmittableMode::Taiko => "accuracy, difficulty",
                SubmittableMode::Mania => "difficulty",
            }
        ))
        .push_values(
            &performance_information,
            |mut b, (performance, (.., score_id))| {
                b.push_bind(BigDecimal::from(**score_id))
                    .push_bind(performance.pp());

                match performance {
                    PerformanceAttributes::Osu(o) => b
                        .push_bind(o.pp_aim)
                        .push_bind(o.pp_speed)
                        .push_bind(o.pp_flashlight)
                        .push_bind(o.pp_acc),
                    PerformanceAttributes::Taiko(t) => {
                        b.push_bind(t.pp_acc).push_bind(t.pp_difficulty)
                    }
                    PerformanceAttributes::Mania(c) => b.push_bind(c.pp_difficulty),
                    PerformanceAttributes::Catch(_) => todo!(),
                };
            },
        )
        .build()
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        locker_guard.unlock().await?;

        Ok(())
    }
}
