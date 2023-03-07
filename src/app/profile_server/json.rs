use serde::{Deserialize, Serialize};

use super::xml::EquippedItemXml;

#[derive(Serialize, Deserialize)]
pub struct Loadout {
    pub slots: Vec<EquippedItem>,
}

impl Loadout {
    pub fn new(equipped_items: &Vec<EquippedItemXml>) -> Self {
        Self { slots: equipped_items.iter()
                                    .map(|i| {
                                        EquippedItem::new(i)
                                    })
                                    .collect()
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
    pub amount: u16
}

impl EquippedItem {
    pub fn new(item: &EquippedItemXml) -> Self {
        Self {
            slot: item.slot,
            index: item.index, key: item.key.to_owned(),
            amount: item.amount
        }
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
    pub amount: u16
}