use super::{item::ItemKind, *};

pub struct Player {
    name: String,
    xp: u64,
    damage: u64,
    gold: u64,
    inventory: RawInventory,
}

impl Player {
    pub fn new(name: &str) -> Self {
        Player {
            name: name.to_owned(),
            ..Default::default()
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn damage(&self) -> u64 {
        self.damage
    }

    pub fn add_damage(&mut self, amount: u64, world: &World) -> u64 {
        let reduced_damage =
            ((amount as f64) * (1.0 - self.damage_reduction(world))).round() as u64;
        self.damage += reduced_damage;
        reduced_damage
    }

    pub fn damage_reduction(&self, world: &World) -> f64 {
        0.12 * self.defense(world) as f64 / 100.0
    }

    pub fn heal(&mut self, amount: u64) {
        self.damage = self.damage.saturating_sub(amount)
    }

    pub fn auto_heal(&mut self, world: &World) -> Option<Vec<String>> {
        let mut potions_used: Vec<String> = Vec::new();

        if self.dead() {
            let mut potions: Vec<&Item> = self
                .inventory_items(world)
                .into_iter()
                .filter(|item| matches!(item.kind(), ItemKind::Potion { .. }))
                .collect();
            potions.sort_by_key(|item| match item.kind() {
                ItemKind::Potion { hp } => hp,
                _ => panic!("There should only ever be potions here."),
            });
            potions.reverse();

            while self.dead() {
                if let Some(potion) = potions.pop() {
                    self.heal(match potion.kind() {
                        ItemKind::Potion { hp } => hp,
                        _ => panic!("There should only ever be potions here."),
                    });
                    self.remove_item(potion.id());
                    potions_used.push(potion.id().to_owned());
                } else {
                    break;
                }
            }
        }

        if potions_used.is_empty() {
            None
        } else {
            Some(potions_used)
        }
    }

    pub fn gold(&self) -> u64 {
        self.gold
    }

    pub fn add_gold(&mut self, amount: u64) {
        self.gold += amount
    }

    pub fn dead(&self) -> bool {
        self.damage > self.hp()
    }

    pub fn net_worth(&self, world: &World) -> u64 {
        let total = self.gold;
        self.inventory_items(world)
            .into_iter()
            .fold(total, |total, item| total + item.value())
    }
}

impl Default for Player {
    fn default() -> Self {
        Player {
            name: String::from("Unnamed"),
            xp: 0,
            damage: 0,
            gold: 0,
            inventory: HashMap::new(),
        }
    }
}

impl Level for Player {
    fn xp(&self) -> u64 {
        self.xp
    }

    fn add_xp(&mut self, amount: u64) {
        self.xp += amount
    }
}

impl Inventory for Player {
    fn inventory(&self) -> &RawInventory {
        &self.inventory
    }

    fn mut_inventory(&mut self) -> &mut RawInventory {
        &mut self.inventory
    }
}
