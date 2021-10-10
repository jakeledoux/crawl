use std::collections::HashMap;

use hottext::{fmt_line, get_line};
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

pub fn enter_cave(world: &mut World, player: &mut Player, context: &mut Context) -> CaveResult {
    // Cave
    let harder_cave_difficulty = CaveDifficulty::random(&mut context.rng);
    let mut caves = vec![
        world.new_cave(player, &mut context.rng, CaveDifficulty::Easy),
        world.new_cave(player, &mut context.rng, harder_cave_difficulty),
    ];
    caves.shuffle(&mut context.rng); // Necessary so that the easy cave isn't always on the left and vice-versa
    let prompt = get_line!(context.hottext, "caves.approach");
    // TODO: Ensure no name collisions
    let (left, right) = (
        get_line!(context.hottext, "caves.names"),
        get_line!(context.hottext, "caves.names"),
    );
    let cave_names: [&str; 2] = [&left, &right];
    let choice = get_choice(context, &prompt, &cave_names);
    let cave = caves.remove(choice);

    context
        .term
        .write_line(&fmt_line!(
            context.hottext,
            "caves.enter",
            cave = cave_names[choice]
        ))
        .unwrap();

    // Increment stats
    world.stats.caves += 1;
    world.stats.monsters += cave.monsters.len() as u64;

    let mut xp = 500; // Minimum cave XP

    if cave.monsters.is_empty() {
        context
            .term
            .write_line(&get_line!(context.hottext, "combat.no-enemies"))
            .unwrap();
    } else {
        for monster in cave.monsters {
            context
                .term
                .write_line(&fmt_line!(
                    context.hottext,
                    "combat.encounter",
                    enemy = monster.name().as_str()
                ))
                .unwrap();

            // Roll for initiative
            let initiative =
                ((player.level().max(1)) as f64 / monster.level() as f64).clamp(0.0, 1.0);
            if context.rng.gen_bool(initiative) {
                context
                    .term
                    .write_line(get_line!(context.hottext, "combat.initiative").as_str())
                    .unwrap();
            } else {
                let damage = monster.damage(&mut context.rng);
                let applied_damage = player.add_damage(damage, world);
                context
                    .term
                    .write_line(&fmt_line!(
                        context.hottext,
                        "combat.attacked",
                        damage = applied_damage.commas().as_str()
                    ))
                    .unwrap();

                // Attempt to heal
                if let Some(potions_used) = player.auto_heal(world) {
                    for potion in potions_used {
                        context
                            .term
                            .write_line(&fmt_line!(
                                context.hottext,
                                "potion.use",
                                damage = world
                                    .get_item(&potion)
                                    .expect("Potion ID pulled directly from world.items",)
                                    .name()
                                    .as_str()
                            ))
                            .unwrap();
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
    context: &mut Context,
) {
    // Loot
    player.add_xp(reward.xp);
    player.add_gold(reward.gold);

    context
        .term
        .write_line(&get_line!(context.hottext, "combat.reward"))
        .unwrap();
    // Generalize reward display
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
    // TODO: "You leveled up!"
    wait_any_key(context);
}

pub fn show_status(world: &World, player: &Player, context: &mut Context) {
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
    wait_any_key(context);
}

pub fn show_inventory(world: &World, player: &Player, context: &mut Context) {
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
    inventory.reverse();

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

    wait_any_key(context);
}

pub fn show_death_screen(world: &World, player: &Player, context: &mut Context) {
    context
        .term
        .write_line(&get_line!(context.hottext, "combat.died"))
        .unwrap();
    context
        .term
        .write_line(&get_line!(context.hottext, "combat.game-over"))
        .unwrap();
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
        player.net_worth(world).commas(),
    );

    let prompt = get_line!(context.hottext, "interface.post-game-menu");
    let choices: [&str; 3] = [
        &get_line!(context.hottext, "interface.view-inventory"),
        &get_line!(context.hottext, "interface.leaderboards"),
        &get_line!(context.hottext, "interface.etxi"),
    ];
    loop {
        let choice_index = get_choice(context, &prompt, &choices);
        match choice_index {
            0 => show_inventory(world, player, context),
            1 => todo!(),
            _ => break,
        }
    }
}
