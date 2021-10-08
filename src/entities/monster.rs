use rand::Rng;
use serde::Deserialize;

use super::Rarity;

#[derive(Deserialize)]
pub struct PotentialMonster {
    pub id: String,
    pub name: String,
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
            generic: self.generic,
            rarity,
            level: rng.gen_range(rarity.level_range()),
        }
    }
}

pub struct Monster {
    name: String,
    generic: bool,
    rarity: Rarity,
    level: u32,
}

impl Monster {
    pub fn name(&self) -> String {
        if self.generic {
            match self.rarity {
                Rarity::Petty => format!("a Petty {}", self.name),
                Rarity::Common => format!("a Common {}", self.name),
                Rarity::Uncommon => format!("an Uncommon {}", self.name),
                Rarity::Rare => format!("a Rare {}", self.name),
                Rarity::Legendary => format!("a Legendary {}", self.name),
            }
        } else {
            self.name.clone()
        }
    }

    pub fn generic(&self) -> bool {
        self.generic
    }

    pub fn rarity(&self) -> Rarity {
        self.rarity
    }

    pub fn level(&self) -> u32 {
        self.level
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
