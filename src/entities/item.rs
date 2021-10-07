use serde::Deserialize;

use super::*;

#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Limb {
    Head,
    Body,
    Hands,
    Feet,
    Shield,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "kind")]
pub enum ItemKind {
    Weapon { damage: u64 },
    Armor { defense: u64, limb: Limb },
    Potion { hp: u64 },
    Collectible,
}

#[derive(Deserialize, Clone)]
pub struct Item {
    id: String,
    #[serde(flatten)]
    kind: ItemKind,
    name: String,
    value: u64,
    #[serde(default = "Rarity::default")]
    rarity: Rarity,
}

impl Item {
    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn value(&self) -> u64 {
        self.value
    }

    pub fn rarity(&self) -> Rarity {
        self.rarity
    }

    pub fn kind(&self) -> ItemKind {
        self.kind
    }
}
