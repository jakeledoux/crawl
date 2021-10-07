// TODO: Make item/enemy rarity scale with player level

#![allow(unused)]

use rand::prelude::*;

use std::error::Error;

mod interface {}
mod entities;

use entities::{player::Player, World, *};

fn main() -> Result<(), Box<dyn Error>> {
    let mut world = World::new()
        .with_load_monsters("monsters/generic.json")?
        .with_load_monsters("monsters/unique.json")?
        .with_load_items("items/armor.json")?
        .with_load_items("items/shields.json")?
        .with_load_items("items/collectibles.json")?
        .with_load_items("items/potions.json")?
        .with_load_items("items/weapons.json")?;
    let mut player = Player::default();
    let mut rng = rand::thread_rng();

    loop {
        // Cave
        let mut caves = vec![world.new_cave(&mut rng), world.new_cave(&mut rng)];
        println!("\nYou approach two caves. Which do you enter?");
        let cave = caves.remove(0);
        println!("You enter the left cave...");

        // Increment stats
        world.stats.caves += 1;
        world.stats.monsters += cave.monsters.len() as u64;

        let mut xp = 100; // Minimum cave XP

        if cave.monsters.is_empty() {
            println!("Sweet, no monsters here!");
        } else {
            for monster in cave.monsters {
                println!("Uh oh, you encounter {}!", monster.name());

                // Roll for initiative
                if rng.gen_bool((player.level() as f64 / monster.level() as f64).clamp(0.0, 1.0)) {
                    println!("You slay the beast before it has a chance to attack!");
                } else {
                    let damage = monster.damage(&mut rng);
                    let applied_damage = player.add_damage(damage, &world);
                    println!("The beast attacks and you take {} damage!", applied_damage);

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
                        xp += monster.level() as u64 * 10
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
        println!("x{} xp", xp);
        println!("x{} gold", cave.gold);
        // TODO: Loot slain monsters
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
        world.stats.caves / 2,
        world.stats.monsters,
        player.gold(),
        player.inventory().values().sum::<u32>()
    );
    println!(
        "That leaves you with a final level of {} and a net worth of {} gold.",
        player.level(),
        player.net_worth(&world)
    );

    Ok(())
}
