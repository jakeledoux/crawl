use rand::Rng;
use serde::Deserialize;

use std::collections::HashMap;
use std::fmt;
use std::ops::Range;

pub mod item;
pub mod monster;
pub mod player;
mod world;

use item::{Item, ItemKind, Limb};
pub use world::*;

const BASE_HP: u64 = 20;

type RawInventory = HashMap<String, u32>;

#[derive(Debug)]
pub struct ItemError {}

impl fmt::Display for ItemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ItemError")
    }
}

impl std::error::Error for ItemError {}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Rarity {
    Petty,
    Common,
    Uncommon,
    Rare,
    Legendary,
}

impl Rarity {
    pub fn random<R>(rng: &mut R) -> Self
    where
        R: Rng,
    {
        match rng.gen_range(0..5) {
            0 => Self::Petty,
            1 => Self::Common,
            2 => Self::Uncommon,
            3 => Self::Rare,
            _ => Self::Legendary,
        }
    }

    pub fn capped_random<R>(rng: &mut R, level: u64) -> Self
    where
        R: Rng,
    {
        match rng.gen_range(0..5.min(level)) {
            0 => Self::Petty,
            1 => Self::Common,
            2 => Self::Uncommon,
            3 => Self::Rare,
            _ => Self::Legendary,
        }
    }

    pub fn level_range(&self) -> Range<u64> {
        match *self {
            Self::Petty => 1..10,
            Self::Common => 10..30,
            Self::Uncommon => 30..100,
            Self::Rare => 100..500,
            Self::Legendary => 500..1000,
        }
    }

    pub fn from_level(level: u64) -> Self {
        match level {
            0..=9 => Self::Petty,
            10..=29 => Self::Common,
            30..=99 => Self::Uncommon,
            100..=499 => Self::Rare,
            _ => Self::Legendary,
        }
    }
}

impl Default for Rarity {
    fn default() -> Rarity {
        Rarity::Common
    }
}

pub trait Inventory {
    fn inventory(&self) -> &RawInventory;

    fn mut_inventory(&mut self) -> &mut RawInventory;

    fn inventory_items<'a, 'b>(&self, world: &'a World) -> Vec<&'b Item>
    where
        'a: 'b,
    {
        self.inventory()
            .iter()
            .flat_map(|(item, amount)| {
                let item = world
                    .get_item(item)
                    .expect("world.items should not have mutated");
                let mut items = Vec::new();
                for _ in 0..*amount {
                    items.push(item);
                }
                items.into_iter()
            })
            .collect()
    }

    fn item_count(&self) -> u32 {
        self.inventory().values().sum()
    }

    fn add_item(&mut self, item: &str) {
        let inventory = self.mut_inventory();
        if let Some(amount) = inventory.get_mut(item) {
            *amount += 1;
        } else {
            inventory.insert(item.to_string(), 1);
        }
    }

    fn has_item(&self, item: &str) -> Option<u32> {
        self.inventory().get(item).copied()
    }

    fn remove_item(&mut self, item: &str) -> Result<(), ItemError> {
        let inventory = self.mut_inventory();
        if let Some(amount) = inventory.get_mut(item) {
            if *amount > 1 {
                *amount -= 1;
            } else {
                inventory.remove(item);
            }
            Ok(())
        } else {
            Err(ItemError {})
        }
    }

    fn defense(&self, world: &World) -> u64 {
        let mut total_defense: HashMap<Limb, u64> = HashMap::new();

        self.inventory_items(world)
            .into_iter()
            .filter(|item| matches!(item.kind(), ItemKind::Armor { .. }))
            .for_each(|item| {
                if let ItemKind::Armor { defense, limb } = item.kind() {
                    match total_defense.get_mut(&limb) {
                        Some(best_defense) => {
                            *best_defense = defense.max(*best_defense);
                        }
                        None => {
                            total_defense.insert(limb, defense);
                        }
                    }
                }
            });

        total_defense.values().sum()
    }
}

pub trait Level {
    fn xp(&self) -> u64;

    fn add_xp(&mut self, amount: u64);

    fn level(&self) -> u64 {
        ((self.xp() as f64).sqrt() / 8.0).round() as u64
    }

    fn hp(&self) -> u64 {
        BASE_HP + self.level() * 5
    }
}
