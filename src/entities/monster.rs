use colored::*;
use rand::Rng;
use serde::Deserialize;

use super::Rarity;
use crate::colors;

#[derive(Deserialize)]
pub struct PotentialMonster {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub proper_noun: bool,
    pub generic: bool,
    pub rarity: Option<Rarity>,
    pub max_level: Option<u64>,
}

impl PotentialMonster {
    pub fn spawn<R>(&self, mut max_rarity: Rarity, rng: &mut R) -> Monster
    where
        R: Rng,
    {
        // Ensure max rarity does not exceed max level
        if let Some(max_level) = self.max_level {
            max_rarity = Rarity::from_level(max_level).min(max_rarity);
        }
        #[allow(clippy::or_fun_call)]
        let rarity = self.rarity.unwrap_or(Rarity::random(rng).min(max_rarity));
        Monster {
            name: self.name.clone(),
            proper_noun: self.proper_noun,
            generic: self.generic,
            rarity,
            level: rng.gen_range(rarity.level_range()),
        }
    }
}

pub struct Monster {
    name: String,
    proper_noun: bool,
    generic: bool,
    rarity: Rarity,
    level: u64,
}

impl Monster {
    fn colored_raw_name(&self) -> String {
        self.name.color(colors::MONSTER).to_string()
    }

    pub fn name(&self) -> String {
        if self.generic {
            match self.rarity {
                Rarity::Petty => format!("Petty {}", self.name),
                Rarity::Common => format!("Common {}", self.name),
                Rarity::Uncommon => format!("Uncommon {}", self.name),
                Rarity::Rare => format!("Rare {}", self.name),
                Rarity::Legendary => format!("Legendary {}", self.name),
            }
        } else {
            self.name.clone()
        }
        .color(colors::MONSTER)
        .to_string()
    }

    pub fn article_name(&self) -> String {
        if self.generic {
            match self.rarity {
                Rarity::Petty => format!("a {}", self.name()),
                Rarity::Common => format!("a {}", self.name()),
                Rarity::Uncommon => format!("an {}", self.name()),
                Rarity::Rare => format!("a {}", self.name()),
                Rarity::Legendary => format!("a {}", self.name()),
            }
        } else {
            self.proper_name()
        }
    }

    pub fn proper_name(&self) -> String {
        if self.proper_noun {
            self.colored_raw_name()
        } else {
            format!("The {}", self.colored_raw_name())
        }
    }

    pub fn generic(&self) -> bool {
        self.generic
    }

    pub fn rarity(&self) -> Rarity {
        self.rarity
    }

    pub fn level(&self) -> u64 {
        self.level
    }

    pub fn is_difficult(&self, player_level: u64) -> bool {
        self.level() / player_level.max(1) > 3 && self.level().saturating_sub(player_level) > 10
    }

    pub fn damage<R>(&self, rng: &mut R) -> u64
    where
        R: Rng,
    {
        let min = (self.level / 10).max(2) as u64;
        let max = (self.level as u64).max(min + 1);
        rng.gen_range(min..max)
    }
}
