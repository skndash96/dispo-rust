use serenity::model::user::User;
use serenity::model::id::ChannelId;
use serenity::client::Context;

use crate::models::HcOptions;

pub async fn start_match (
    ctx: &Context,
    channel_id: ChannelId,
    bat_team: Vec<&User>,
    bowl_team: Vec<&User>,
    options: &HcOptions,
    is_duo: bool
) -> Result<(), String> {
    Ok(())
}
