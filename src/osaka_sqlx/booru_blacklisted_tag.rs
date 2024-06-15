use crate::{commands::booru::SettingKind, osaka_sqlx::Jib, OsakaContext};
use sqlx::types::BigDecimal;
use sqlx_conditional_queries_layering::create_conditional_query_as;

use crate::get_conditional_id_kind_query;

pub struct BooruBlacklistedTag {
    pub blacklisted: String,
    pub booru_setting_id: BigDecimal,
}

#[macro_export]
macro_rules! get_blacklist_query {
    () => {
        create_conditional_query_as!(
            blacklist_query,
            #blacklist_query = match Jib::Jab { Jab =>
            "
            SELECT t.* FROM booru_blacklisted_tag t
            JOIN booru_setting s ON s.id=t.booru_setting_id
            WHERE s.id=t.booru_setting_id
            "
        });
    };
}

#[macro_export]
macro_rules! get_blacklist_for_kind_query {
    () => {
        blacklist_query_feed_existing_query!(conditional_id_kind_query, blacklist_for_kind_query);
    };
}

get_blacklist_query!();
impl BooruBlacklistedTag {
    fn map_to_string(tags: Result<Vec<BooruBlacklistedTag>, sqlx::Error>) -> Vec<String> {
        tags.map(|v| v.iter().map(|v| v.blacklisted.clone()).collect())
            .unwrap_or_default()
    }

    pub async fn fetch_all(ctx: OsakaContext<'_>) -> Vec<String> {
        Self::map_to_string(
            blacklist_query!(BooruBlacklistedTag, "{#blacklist_query}")
                .fetch_all(&ctx.data().pool)
                .await,
        )
    }

    pub async fn fetch_all_for_kind(ctx: OsakaContext<'_>, kind: SettingKind) -> Vec<String> {
        let inserted_discord_id = kind.get_sqlx_id(ctx).unwrap_or_default();

        get_conditional_id_kind_query!(kind);
        get_blacklist_for_kind_query!();

        Self::map_to_string(
            blacklist_for_kind_query!(
                BooruBlacklistedTag,
                "
                {#blacklist_query}
                AND s.{#id_kind}_id={inserted_discord_id}
                ",
            )
            .fetch_all(&ctx.data().pool)
            .await,
        )
    }
}