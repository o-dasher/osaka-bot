use poise::ChoiceParameter;

use crate::{
    error::OsakaError, osaka_sqlx::booru_setting::SettingKind, OsakaContext, OsakaData, OsakaResult,
};

#[derive(ChoiceParameter, Default, Clone, Copy)]
#[repr(u8)]
pub enum OsuMode {
    #[default]
    #[name = "osu"]
    Osu = 0,

    #[name = "taiko"]
    Taiko = 1,

    #[name = "catch"]
    Catch = 2,

    #[name = "mania"]
    Mania = 3,
}

#[poise::command(slash_command)]
pub async fn submit(ctx: OsakaContext<'_>, mode: OsuMode) -> OsakaResult {
    todo!();

    let OsakaData { pool, .. } = ctx.data();

    let user = sqlx::query!(
        "SELECT * FROM discord_user WHERE id=$1",
        SettingKind::User.get_sqlx_id(ctx)?
    )
    .fetch_one(pool)
    .await
    .map_err(|_| OsakaError::SimplyUnexpected)?;

    Ok(())
}