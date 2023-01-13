use serenity::framework::standard::macros::*;

mod rank;
use rank::RANK_COMMAND;

mod hc;
use hc::HC_COMMAND;

#[group]
#[commands(hc, rank)]
#[description = "All handcricket related commands."]
#[help_available]
pub struct Cricket;
