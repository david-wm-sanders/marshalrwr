use serde::{Deserialize, Serialize};

use super::xml::{EntryXml, EquippedItemXml, MonitorXml, StoredItemXml};

#[derive(Serialize, Deserialize)]
pub struct Loadout {
    pub slots: Vec<EquippedItem>,
}

impl Loadout {
    pub fn new(equipped_items: &[EquippedItemXml]) -> Self {
        Self {
            slots: equipped_items
                .iter()
                .map(|i| EquippedItem::new(i))
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct EquippedItem {
    #[serde(rename = "s")]
    pub slot: u8,
    #[serde(rename = "i")]
    pub index: i32,
    #[serde(rename = "k")]
    pub key: String,
    #[serde(rename = "a")]
    pub amount: u16,
}

impl EquippedItem {
    pub fn new(item: &EquippedItemXml) -> Self {
        Self {
            slot: item.slot,
            index: item.index,
            key: item.key.to_owned(),
            amount: item.amount,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ItemStore {
    pub items: Vec<StoredItem>,
}

impl ItemStore {
    pub fn new(stored_items: &[StoredItemXml]) -> Self {
        let v: Vec<StoredItem> = stored_items
            .iter()
            .map(|item| StoredItem {
                class: item.class,
                index: item.index,
                key: item.key.to_owned(),
                amount: item.amount,
            })
            .collect();
        Self { items: v }
    }
}

#[derive(Serialize, Deserialize)]
pub struct StoredItem {
    #[serde(rename = "c")]
    pub class: u8,
    #[serde(rename = "i")]
    pub index: i32,
    #[serde(rename = "k")]
    pub key: String,
    #[serde(rename = "a")]
    pub amount: u16,
}

pub type KillCombo = (i32, i32);

#[derive(Serialize, Deserialize)]
pub struct KillCombos {
    pub entries: Vec<KillCombo>,
}

impl KillCombos {
    pub fn new(entries: &[EntryXml]) -> Self {
        Self {
            entries: entries
                .iter()
                .map(|entry| (entry.combo, entry.count))
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CriteriaMonitors {
    pub monitors: Vec<CriteriaMonitor>,
}

#[derive(Serialize, Deserialize)]
pub struct CriteriaMonitor {
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "l")]
    pub level: i32,
    #[serde(rename = "c")]
    pub critera: Vec<i32>,
}

impl CriteriaMonitor {
    pub fn new(criteria_monitor: &MonitorXml) -> Self {
        Self {
            name: criteria_monitor.name.to_owned().unwrap_or_default(),
            level: criteria_monitor.level.unwrap_or(0),
            critera: criteria_monitor.criteria.iter().map(|c| c.count).collect(),
        }
    }
}
