use std::collections::HashMap;

use console::Term;
use rand::prelude::*;

use super::entities::{player::Player, CaveDifficulty, World, *};
use super::interface::*;

pub struct CaveReward {
    pub xp: u64,
    pub gold: u64,
    pub loot: HashMap<String, u32>,
}

pub enum CaveResult {
    Survived { reward: CaveReward },
    Died,
}

pub fn enter_cave<R>(
    world: &mut World,
    player: &mut Player,
    rng: &mut R,
    term: &mut Term,
) -> CaveResult
where
    R: Rng,
{
    // Cave
    let harder_cave_difficulty = CaveDifficulty::random(rng);
    let mut caves = vec![
        world.new_cave(&player, rng, CaveDifficulty::Easy),
        world.new_cave(&player, rng, harder_cave_difficulty),
    ];
    caves.shuffle(rng); // Necessary so that the easy cave isn't always on the left and vice-versa
    let choice = get_choice(
        term,
        "You approach two caves. Which do you enter?",
        &["left", "right"],
    );
    let cave = caves.remove(choice);
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
                let damage = monster.damage(rng);
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
                                .name(),
                        );
                    }
                }
                if player.dead() {
                    return CaveResult::Died;
                } else {
                    println!(
                        "You power through the pain! ({} HP remaining)",
                        player.hp().saturating_sub(player.damage()),
                    );
                    xp += monster.level() as u64 * 20
                }
            }
        }
    }

    let reward = CaveReward {
        xp,
        gold: cave.gold,
        loot: cave.loot,
    };
    CaveResult::Survived { reward }
}

pub fn show_cave_reward(
    world: &mut World,
    player: &mut Player,
    reward: CaveReward,
    term: &mut Term,
) {
    // Loot
    player.add_xp(reward.xp);
    player.add_gold(reward.gold);
    println!("You got...");
    println!("x{} xp", reward.xp.commas());
    println!("x{} gold", reward.gold.commas());
    reward.loot.into_iter().for_each(|(item, count)| {
        for _ in 0..count {
            player.add_item(&item);
        }
        println!(
            "x{} {}",
            count,
            world
                .get_item(&item)
                .expect("world.items should not have mutated")
                .name(),
        );
    });
    wait_any_key(term);
}

pub fn show_status(world: &World, player: &Player, term: &mut Term) {
    println!(
        "Level {} ({} xp), {}/{} hp, {} gold, {} items, {} armor",
        player.level().commas(),
        player.xp().commas(),
        player.hp_remaining(),
        player.hp(),
        player.gold(),
        player.item_count().commas(),
        player.defense(world).commas(),
    );
    wait_any_key(term);
}

pub fn show_inventory(world: &World, player: &Player, term: &mut Term) {
    let mut inventory = player
        .inventory()
        .iter()
        .map(|(item, count)| {
            (
                world
                    .get_item(item)
                    .expect("world.items should not have mutated")
                    .name()
                    .as_ref(),
                *count,
            )
        })
        .collect::<Vec<(&str, u32)>>();
    inventory.sort_by_key(|e| e.1);

    let show_item = |(name, count): (&str, u32)| {
        println!(
            "x{:<width$} - {}",
            count.commas(),
            name,
            width = player.gold().commas().len()
        );
    };
    show_item(("gold", player.gold() as u32));
    inventory.into_iter().for_each(show_item);

    wait_any_key(term);
}

pub fn show_death_screen(world: &World, player: &Player, term: &mut Term) {
    println!("\nYou have died. Game over.");
    println!("\nOver the course of your journey you encountered:");
    println!(
        "{} caves, {} monsters, {} gold, and {} items.",
        (world.stats.caves / 2).commas(),
        world.stats.monsters.commas(),
        player.gold().commas(),
        player.item_count().commas(),
    );
    println!(
        "That leaves you with a final level of {} ({} xp) and a net worth of {} gold.",
        player.level().commas(),
        player.xp().commas(),
        player.net_worth(&world).commas(),
    );

    loop {
        let choices = ["view inventory", "leaderboards", "exit"];
        let choice_index = get_choice(term, "What do you want to do?", &choices);
        match choice_index {
            0 => show_inventory(world, player, term),
            1 => todo!(),
            _ => break,
        }
    }
}
