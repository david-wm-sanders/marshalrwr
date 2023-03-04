use serde::Deserialize;
use validator::Validate;

use super::validation::{validate_username, RE_HEX_STR};

// todo: validate that hash for player username

#[derive(Debug, Deserialize, Validate)]
pub struct SetProfileDataXml {
    #[serde(rename = "player")]
    #[validate]
    pub players: Vec<PlayerXml>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PlayerXml {
    #[serde(rename = "@hash")]
    #[validate(range(min=1, max="u32::MAX"))]
    pub hash: i64,
    #[serde(rename = "@rid")]
    #[validate(length(equal=64))]
    #[validate(regex(path="RE_HEX_STR", code="rid not hexadecimal"))]
    pub rid: String,
    #[validate]
    pub person: PersonXml,
    #[validate]
    pub profile: ProfileXml,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PersonXml {
    #[serde(rename = "@max_authority_reached")]
    pub max_authority_reached: f32,
    #[serde(rename = "@authority")]
    pub authority: f32,
    #[serde(rename = "@job_points")]
    pub job_points: f32,
    #[serde(rename = "@faction")]
    pub faction: i32,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@soldier_group_id")]
    pub soldier_group_id: i32,
    #[serde(rename = "@soldier_group_name")]
    pub soldier_group_name: String,
    #[serde(rename = "@squad_size_setting")]
    pub squad_size_setting: i32,
    // todo: add loadout, stash, and backpack
    #[serde(rename = "item")]
    pub equipped_items: Vec<EquippedItemXml>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct EquippedItemXml {
    #[serde(rename = "@slot")]
    pub slot: i32,
    #[serde(rename = "@index")]
    pub index: i32,
    #[serde(rename = "@amount")]
    pub amount: i32,
    #[serde(rename = "@key")]
    pub key: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ProfileXml {
    #[serde(rename = "@game_version")]
    pub game_version: i32,
    #[serde(rename = "@username")]
    #[validate(length(min=1, max=32))]
    #[validate(non_control_character)]
    #[validate(custom(function="validate_username"))]
    pub username: String,
    #[serde(rename = "@sid")]
    #[validate(range(min=1, max="u32::MAX"))]
    pub sid: i64,
    #[serde(rename = "@rid")]
    #[validate(length(equal=64))]
    #[validate(regex(path="RE_HEX_STR", code="rid not hexadecimal"))]
    pub rid: String,
    #[serde(rename = "@squad_tag")]
    #[validate(length(min=0, max=3))]
    pub squad_tag: String,
    pub stats: StatsXml,
}

#[derive(Debug, Deserialize, Validate)]
pub struct StatsXml {
    #[serde(rename = "@kills")]
    pub kills: i32,
    #[serde(rename = "@deaths")]
    pub deaths: i32,
    #[serde(rename = "@time_played")]
    pub time_played: f32,
    #[serde(rename = "@player_kills")]
    pub player_kills: i32,
    #[serde(rename = "@teamkills")]
    pub teamkills: i32,
    #[serde(rename = "@longest_kill_streak")]
    pub longest_kill_streak: i32,
    #[serde(rename = "@targets_destroyed")]
    pub targets_destroyed: i32,
    #[serde(rename = "@vehicles_destroyed")]
    pub vehicles_destroyed: i32,
    #[serde(rename = "@soldiers_healed")]
    pub soldiers_healed: i32,
    #[serde(rename = "@distance_moved")]
    pub distance_moved: f32,
    #[serde(rename = "@shots_fired")]
    pub shots_fired: i32,
    #[serde(rename = "@throwables_thrown")]
    pub throwables_thrown: i32,
    #[serde(rename = "@rank_progression")]
    pub rank_progression: f32,
    // todo: add monitors
}