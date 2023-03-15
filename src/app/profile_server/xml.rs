use std::sync::Arc;

use serde::{Deserialize, Serialize};
use validator::Validate;

use super::{
    errors::ProfileServerError,
    json::{CriteriaMonitors, ItemStore, KillCombos, Loadout},
    validation::{validate_username, RE_HEX_STR},
};
use entity::{AccountModel, PlayerModel};

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
    #[validate(range(min = 1, max = "u32::MAX"))]
    pub hash: i64,
    #[serde(rename = "@rid")]
    #[validate(length(equal = 64))]
    #[validate(regex(path = "RE_HEX_STR", code = "rid not hexadecimal"))]
    pub rid: String,
    #[validate]
    pub person: PersonXml,
    #[validate]
    pub profile: ProfileXml,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
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
    #[serde(rename = "@version")]
    pub version: i32, /* version must be provided for items and item_groups to be handled properly */
    #[serde(rename = "@soldier_group_id")]
    pub soldier_group_id: i32,
    #[serde(rename = "@soldier_group_name")]
    pub soldier_group_name: String,
    #[serde(rename = "@squad_size_setting")]
    pub squad_size_setting: i32,
    #[serde(rename = "item")]
    pub equipped_items: Vec<EquippedItemXml>,
    pub backpack: ItemStoreXml,
    pub stash: ItemStoreXml,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct EquippedItemXml {
    #[serde(rename = "@slot")]
    pub slot: u8,
    #[serde(rename = "@index")]
    pub index: i32,
    #[serde(rename = "@amount")]
    pub amount: u16,
    #[serde(rename = "@key")]
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ItemStoreXml {
    #[serde(rename = "item_group")]
    #[serde(default)]
    pub items: Vec<StoredItemXml>,
}

impl ItemStoreXml {
    pub fn new(item_store: &ItemStore) -> Self {
        Self {
            items: item_store
                .items
                .iter()
                .map(|item| StoredItemXml {
                    class: item.class,
                    index: item.index,
                    key: item.key.to_owned(),
                    amount: item.amount,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct StoredItemXml {
    #[serde(rename = "@class")]
    pub class: u8,
    #[serde(rename = "@index")]
    pub index: i32,
    #[serde(rename = "@key")]
    pub key: String,
    #[serde(rename = "@amount")]
    pub amount: u16,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ProfileXml {
    #[serde(rename = "@game_version")]
    pub game_version: i32,
    #[serde(rename = "@username")]
    #[validate(length(min = 1, max = 32))]
    #[validate(non_control_character)]
    #[validate(custom(function = "validate_username"))]
    pub username: String,
    #[serde(rename = "@sid")]
    #[validate(range(min = 1, max = "u32::MAX"))]
    pub sid: i64,
    #[serde(rename = "@rid")]
    #[validate(length(equal = 64))]
    #[validate(regex(path = "RE_HEX_STR", code = "rid not hexadecimal"))]
    pub rid: String,
    #[serde(rename = "@squad_tag")]
    #[validate(length(min = 0, max = 3))]
    pub squad_tag: String,
    pub stats: StatsXml,
}

#[derive(Debug, Default, Serialize, Deserialize, Validate)]
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
    // monitors
    #[serde(rename = "monitor")]
    #[serde(default)]
    pub monitors: Vec<MonitorXml>,
}

#[derive(Debug, Default, Serialize, Deserialize, Validate)]
pub struct MonitorXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@longest_death_streak")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longest_death_streak: Option<i32>,
    #[serde(rename = "@level")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<i32>,
    #[serde(rename = "entry")]
    #[serde(default)]
    pub entries: Vec<EntryXml>,
    #[serde(rename = "criteria")]
    #[serde(default)]
    pub criteria: Vec<CriteriaXml>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntryXml {
    #[serde(rename = "@combo")]
    pub combo: i32,
    #[serde(rename = "@count")]
    pub count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CriteriaXml {
    #[serde(rename = "@count")]
    pub count: i32,
}

#[derive(Debug, Serialize)]
pub struct GetProfileDataXml {
    #[serde(rename = "@ok")]
    ok: i32,
    profile: ProfileXml,
    person: PersonXml,
}

impl GetProfileDataXml {
    pub fn new(
        player: &Arc<PlayerModel>,
        account: &Arc<AccountModel>,
    ) -> Result<Self, ProfileServerError> {
        let loadout_json: Loadout = serde_json::from_str(&account.loadout)?;
        let backpack_json: ItemStore = serde_json::from_str(&account.backpack)?;
        let stash_json: ItemStore = serde_json::from_str(&account.stash)?;
        // construct monitors
        let kill_combos_json: KillCombos = serde_json::from_str(&account.kill_combos)?;
        let criteria_monitors_json: CriteriaMonitors =
            serde_json::from_str(&account.criteria_monitors)?;

        let mut monitors = Vec::new();
        // insert the longest death streak monitor
        monitors.push(MonitorXml {
            name: Some("death streak".to_string()),
            longest_death_streak: Some(account.longest_death_streak),
            level: None,
            ..Default::default()
        });
        // insert the kill combos monitor
        monitors.push(MonitorXml {
            name: Some("kill combo".to_string()),
            entries: kill_combos_json
                .entries
                .iter()
                .map(|e| EntryXml {
                    combo: e.0,
                    count: e.1,
                })
                .collect(),
            ..Default::default()
        });
        // insert the criteria monitors
        for cm in criteria_monitors_json.monitors.iter() {
            monitors.push(MonitorXml {
                name: Some(cm.name.to_owned()),
                level: Some(cm.level),
                criteria: cm
                    .critera
                    .iter()
                    .map(|c| CriteriaXml { count: *c })
                    .collect(),
                ..Default::default()
            });
        }

        Ok(Self {
            ok: 1,
            person: PersonXml {
                max_authority_reached: account.max_authority_reached as f32,
                authority: account.authority as f32,
                job_points: account.job_points as f32,
                faction: account.faction,
                name: account.name.clone(),
                version: account.game_version,
                soldier_group_id: account.soldier_group_id,
                soldier_group_name: account.soldier_group_name.clone(),
                squad_size_setting: account.squad_size_setting,
                equipped_items: loadout_json
                    .slots
                    .iter()
                    .map(|item| EquippedItemXml {
                        slot: item.slot,
                        index: item.index,
                        key: item.key.to_owned(),
                        amount: item.amount,
                    })
                    .collect(),
                backpack: ItemStoreXml::new(&backpack_json),
                stash: ItemStoreXml::new(&stash_json),
            },
            profile: ProfileXml {
                game_version: account.game_version,
                username: player.username.clone(),
                sid: player.sid,
                rid: player.rid.clone(),
                squad_tag: account.squad_tag.clone(),
                stats: StatsXml {
                    kills: account.kills,
                    deaths: account.deaths,
                    time_played: account.time_played as f32,
                    player_kills: account.player_kills,
                    teamkills: account.teamkills,
                    longest_kill_streak: account.longest_kill_streak,
                    targets_destroyed: account.targets_destroyed,
                    vehicles_destroyed: account.vehicles_destroyed,
                    soldiers_healed: account.soldiers_healed,
                    distance_moved: account.distance_moved as f32,
                    shots_fired: account.shots_fired,
                    throwables_thrown: account.throwables_thrown,
                    rank_progression: account.rank_progression as f32,
                    monitors,
                },
            },
        })
    }
}
