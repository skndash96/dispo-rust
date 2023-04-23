use serenity::model::id::ChannelId;
use serenity::client::Context;

use crate::models::{
    HcOptions,
    HcPlayer
};

pub async fn start_match<'a> (
    ctx: &Context,
    channel_id: ChannelId,
    bat_team: Vec<HcPlayer<'a>>,
    bowl_team: Vec<HcPlayer<'a>>,
    options: HcOptions,
    is_duo: bool
) -> Result<(), String> {
    //dududjdjdjdjfjfjdjdjxjxjcjcjfjkfkffkfkckdkdkdkfkdkkf
    Ok(())
}
