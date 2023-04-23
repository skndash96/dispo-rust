use std::collections::HashMap;

use serde::{ Serialize, Deserialize };

use serenity::model::user::User;
use serenity::prelude::TypeMapKey;

use mongodb::bson::DateTime;
use mongodb::Collection;

#[derive(Debug, Copy, Clone)]
pub struct Extra<'a> (
    pub Option<&'a User>
);

#[derive(Debug, Copy, Clone)]
pub enum HcPlayer<'a> {
    U (&'a User),
    E (Extra<'a>)
}

pub type EmojisType = HashMap<String, String>;

pub struct Emojis;
impl TypeMapKey for Emojis {
    type Value = EmojisType;
}

pub type EngagedType = Vec<String>;

pub struct Engaged;
impl TypeMapKey for Engaged {
    type Value = EngagedType;
}

#[derive(Debug)]
pub struct HcOptions {
    pub post: bool,
    pub wickets: u8,
    pub overs: u8
}

pub type CommandsInfo = HashMap<String, Vec<String>>;

pub struct Commands;
impl TypeMapKey for Commands {
    type Value = CommandsInfo;
}

#[derive(Debug)]
pub struct Players;
impl TypeMapKey for Players {
    type Value = Collection::<Player>;
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    //Any player who overflows the type shall be rewarded nastily.
    pub _id: String,
    pub cc: Option<u64>,
    pub xp: Option<f32>,
    pub stamina: Option<u8>,
    pub status: Option<bool>,
    pub start: Option<DateTime>,
    pub strikeRate: Option<f32>,
    pub orangeCaps: Option<u16>,
    pub ducksTaken: Option<u16>,
    pub highScore: Option<u32>,
    pub totalScore: Option<u32>,
    pub wickets: Option<u32>,
    pub wins: Option<u32>,
    pub loses: Option<u32>,
    pub coinMulti: Option<f32>,
    pub tossMulti: Option<f32>,
    pub coinBoost: Option<DateTime>,
    pub tossBoost: Option<DateTime>,
}
/*
challengeProgress,
pattern,
bag,
decors,
cards,
voteClaim,
voteCooldown,
voteStreak,
lastVoted,
*/