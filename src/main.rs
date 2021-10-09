// TODO: Make item/enemy rarity scale with player level

#![allow(unused)]

use rand::prelude::*;

use std::error::Error;

mod interface {
    use std::fmt::Display;

    pub trait Comma
    where
        Self: std::fmt::Display,
    {
        /// Formats number with commas separating every three decimal places
        /// # Examples
        /// ```
        /// let s = 6000.commas();
        /// assert_eq(s, "6,000");
        /// ```
        fn commas(&self) -> String {
            let raw_display = format!("{}", self);
            raw_display
                .chars()
                .rev()
                .fold((String::new(), 0), |(mut output, mut length), char| {
                    length += 1;
                    if length % 3 == 1 && length >= 3 {
                        output.push(',');
                    }
                    output.push(char);

                    (output, length)
                })
                .0
                .chars()
                .rev()
                .collect()
        }
    }

    impl Comma for u8 {}
    impl Comma for u16 {}
    impl Comma for u32 {}
    impl Comma for u64 {}
    impl Comma for i8 {}
    impl Comma for i16 {}
    impl Comma for i32 {}
    impl Comma for i64 {}
}
mod entities;

use entities::{player::Player, CaveDifficulty, World, *};
use interface::Comma;

fn main() -> Result<(), Box<dyn Error>> {
    let mut world = World::new()
        .with_load_monsters("data/monsters/generic.json")?
        .with_load_monsters("data/monsters/unique.json")?
        .with_load_items("data/items/armor.json")?
        .with_load_items("data/items/shields.json")?
        .with_load_items("data/items/collectibles.json")?
        .with_load_items("data/items/potions.json")?
        .with_load_items("data/items/weapons.json")?;
    let mut player = Player::default();
    let mut rng = rand::thread_rng();

    loop {
        // Cave
        let harder_cave_difficulty = CaveDifficulty::random(&mut rng);
        let mut caves = vec![
            world.new_cave(&player, &mut rng, CaveDifficulty::Easy),
            world.new_cave(&player, &mut rng, harder_cave_difficulty),
        ];
        caves.shuffle(&mut rng);
        println!("\nYou approach two caves. Which do you enter?");
        let cave = caves.remove(0);
        println!("You enter the left cave...");

        // Increment stats
        world.stats.caves += 1;
        world.stats.monsters += cave.monsters.len() as u64;

        let mut xp = 500; // Minimum cave XP

        if cave.monsters.is_empty() {
            println!("Sweet, no monsters here!");
        } else {
            for monster in cave.monsters {
                println!("Uh oh, you encounter {}!", monster.name());

                // Roll for initiative
                let initiative =
                    ((player.level().max(1)) as f64 / monster.level() as f64).clamp(0.0, 1.0);
                if rng.gen_bool(initiative) {
                    println!("You slay it before it has a chance to attack!");
                } else {
                    let damage = monster.damage(&mut rng);
                    let applied_damage = player.add_damage(damage, &world);
                    println!("It attacks and you take {} damage!", applied_damage);

                    // Attempt to heal
                    if let Some(potions_used) = player.auto_heal(&world) {
                        for potion in potions_used {
                            println!(
                                "You used {}",
                                world
                                    .get_item(&potion)
                                    .expect("Potion ID pulled directly from world.items")
                                    .name()
                            );
                        }
                    }
                    if player.dead() {
                        break;
                    } else {
                        println!(
                            "You power through the pain! ({} HP remaining)",
                            player.hp().saturating_sub(player.damage())
                        );
                        xp += monster.level() as u64 * 20
                    }
                }
            }
            if player.dead() {
                break;
            }
        }

        // Loot
        player.add_xp(xp);
        player.add_gold(cave.gold);
        println!("You got...");
        println!("x{} xp", xp.commas());
        println!("x{} gold", cave.gold);
        cave.loot.into_iter().for_each(|(item, count)| {
            for _ in 0..count {
                player.add_item(&item);
            }
            println!(
                "x{} {}",
                count,
                world
                    .get_item(&item)
                    .expect("world.items should not have mutated")
                    .name()
            );
        });
    }

    // Game over
    println!("\nYou have died. Game over.");
    println!("\nOver the course of your journey you encountered:");
    println!(
        "{} caves, {} monsters, {} gold, and {} items.",
        (world.stats.caves / 2).commas(),
        world.stats.monsters.commas(),
        player.gold().commas(),
        player.inventory().values().sum::<u32>().commas()
    );
    println!(
        "That leaves you with a final level of {} ({} xp) and a net worth of {} gold.",
        player.level().commas(),
        player.xp().commas(),
        player.net_worth(&world).commas()
    );

    Ok(())
}
