use itertools::Itertools;
use poise::command;

use crate::{
    osaka_sqlx::{booru_blacklisted_tag::BooruBlacklistedTag, booru_setting::SettingKind},
    responses::markdown::mono,
    utils::pagination::Paginator,
    OsakaContext, OsakaResult,
};

#[command(slash_command)]
pub async fn list(ctx: OsakaContext<'_>, kind: SettingKind) -> OsakaResult {
    let result = BooruBlacklistedTag::fetch_all_for_kind(ctx, kind).await;
    let chunk_result = result
        .iter()
        .chunks(64)
        .into_iter()
        .map(Itertools::collect_vec)
        .collect_vec();

    Paginator::new(ctx, chunk_result.len())
        .paginate(|idx, r| {
            dbg!(idx);

            Ok(r.embed(|e| {
                e.title(format!("Blacklist for {kind}")).description({
                    if let Some(idx_values) = chunk_result.get(idx)
                        && !idx_values.is_empty()
                    {
                        format!("{}.", idx_values.iter().map(mono).join(", "))
                    } else {
                        "No blacklists here...".to_string()
                    }
                })
            })
            .to_owned())
        })
        .await?;

    Ok(())
}
