use rand::Rng;
use serde::Deserialize;

use super::Rarity;

#[derive(Deserialize)]
pub struct PotentialMonster {
    pub id: String,
    pub name: String,
    pub generic: bool,
    pub rarity: Option<Rarity>,
}

impl PotentialMonster {
    pub fn spawn<R>(&self, rng: &mut R) -> Monster
    where
        R: Rng,
    {
        #[allow(clippy::or_fun_call)]
        let rarity = self.rarity.unwrap_or(Rarity::capped_random(rng, 3));
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
        dbg!(min..max);
        rng.gen_range(min..max)
    }
}
