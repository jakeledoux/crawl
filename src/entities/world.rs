use rand::prelude::*;

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::path::Path;

use super::{
    item::Item,
    monster::{Monster, PotentialMonster},
    Inventory, RawInventory,
};

pub struct Stats {
    pub caves: u64,
    pub monsters: u64,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            caves: 0,
            monsters: 0,
        }
    }
}

pub struct Cave {
    pub loot: RawInventory,
    pub gold: u64,
    pub monsters: Vec<Monster>,
    // TODO: name: String
    // TODO: kind: CaveKind
}

impl Inventory for Cave {
    fn inventory(&self) -> &RawInventory {
        &self.loot
    }

    fn mut_inventory(&mut self) -> &mut RawInventory {
        &mut self.loot
    }
}

pub struct World {
    items: HashMap<String, Item>,
    monsters: HashMap<String, PotentialMonster>,
    pub stats: Stats,
}

impl World {
    pub fn new() -> Self {
        World {
            items: HashMap::new(),
            monsters: HashMap::new(),
            stats: Stats::default(),
        }
    }

    pub fn items(&self) -> &HashMap<String, Item> {
        &self.items
    }

    pub fn item_ids(&self) -> Vec<String> {
        self.items.keys().cloned().collect()
    }

    pub fn get_item(&self, item: &str) -> Option<&Item> {
        self.items.get(item)
    }

    pub fn load_items<P: AsRef<Path>>(&mut self, file: P) -> Result<u32, Box<dyn Error>> {
        let file = File::open(file)?;
        let items: Vec<Item> = serde_json::from_reader(file)?;

        Ok(items.into_iter().fold(0, |total, item| {
            if !self.items.contains_key(item.id()) {
                self.items.insert(item.id().into(), item);
                total + 1
            } else {
                total
            }
        }))
    }

    pub fn with_load_items<P: AsRef<Path>>(mut self, file: P) -> Result<Self, Box<dyn Error>> {
        self.load_items(file)?;
        Ok(self)
    }

    pub fn load_monsters<P: AsRef<Path>>(&mut self, file: P) -> Result<u32, Box<dyn Error>> {
        let file = File::open(file)?;
        let monsters: Vec<PotentialMonster> = serde_json::from_reader(file)?;

        Ok(monsters.into_iter().fold(0, |total, monster| {
            if !self.monsters.contains_key(&monster.id) {
                self.monsters.insert(monster.id.clone(), monster);
                total + 1
            } else {
                total
            }
        }))
    }

    pub fn with_load_monsters<P: AsRef<Path>>(mut self, file: P) -> Result<Self, Box<dyn Error>> {
        self.load_monsters(file)?;
        Ok(self)
    }

    pub fn new_cave<R>(&mut self, rng: &mut R) -> Cave
    where
        R: Rng,
    {
        // Generate loot
        let loot_count = rng.gen_range(1..5);
        let mut loot = self.item_ids();
        // TODO: use sample_iter() to allow for duplicate loot
        loot.shuffle(rng);
        let loot = loot
            .into_iter()
            .take(loot_count)
            .map(|item| (item, 1))
            .collect();
        // Generate gold
        let gold = rng.gen_range(0..200);
        // Generate monsters
        let mut monsters = Vec::new();
        let monster_count = (rng.gen_range(0.0..1000_f64).sqrt() / 10.0) as u32;
        for _ in 0..monster_count {
            monsters.push(
                self.monsters
                    .values()
                    .choose(rng)
                    .expect("Monsters will not be empty.")
                    .spawn(rng),
            );
        }

        Cave {
            loot,
            gold,
            monsters,
        }
    }
}
