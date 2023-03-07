use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Loadout {
    slots: Vec<EquippedItem>,
}

#[derive(Serialize, Deserialize)]
struct EquippedItem {
    #[serde(rename = "s")]
    slot: u8,
    #[serde(rename = "i")]
    index: i32,
    #[serde(rename = "k")]
    key: String,
    #[serde(rename = "a")]
    amount: u8
}

#[derive(Serialize, Deserialize)]
struct StoredItem {
    #[serde(rename = "c")]
    class: u8,
    #[serde(rename = "i")]
    index: i32,
    #[serde(rename = "k")]
    key: String
}