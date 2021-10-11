use std::collections::HashMap;

use colored::*;
use hottext::{fmt_line, get_line, get_lines};
use rand::prelude::*;

use super::entities::{player::Player, CaveDifficulty, World, *};
use super::interface::*;
use crate::colors;

pub struct CaveReward {
    pub xp: u64,
    pub gold: u64,
    pub loot: HashMap<String, u32>,
}

pub enum CaveResult {
    Survived { reward: CaveReward },
    Died,
}

pub enum CaveSurvivedChoice {
    ShowStatusReport,
    Continue,
}

impl CaveSurvivedChoice {
    /// Returns `true` if the cave_survived_choice is [`ShowStatusReport`].
    pub fn is_show_status_report(&self) -> bool {
        matches!(self, Self::ShowStatusReport)
    }

    /// Returns `true` if the cave_survived_choice is [`Continue`].
    pub fn is_continue(&self) -> bool {
        matches!(self, Self::Continue)
    }
}

pub enum GameOverChoice {
    Retry,
    Quit,
}

impl GameOverChoice {
    /// Returns `true` if the game_over_choice is [`Retry`].
    pub fn is_retry(&self) -> bool {
        matches!(self, Self::Retry)
    }

    /// Returns `true` if the game_over_choice is [`Quit`].
    pub fn is_quit(&self) -> bool {
        matches!(self, Self::Quit)
    }
}

pub fn enter_cave(world: &mut World, player: &mut Player, ctx: &mut Context) -> CaveResult {
    spacer(ctx);

    // Cave
    let harder_cave_difficulty = CaveDifficulty::random(&mut ctx.rng);
    let mut caves = vec![
        world.new_cave(player, &mut ctx.rng, CaveDifficulty::Easy),
        world.new_cave(player, &mut ctx.rng, harder_cave_difficulty),
    ];
    caves.shuffle(&mut ctx.rng); // Ensure random spacial distribution of hard caves
    let prompt = get_line!(ctx.hottext, "caves.approach");
    let cave_names = get_lines!(ctx.hottext, "caves.names")
        .into_iter()
        .choose_multiple(&mut ctx.rng, 2);
    let choice = get_choice(
        ctx,
        &prompt,
        &cave_names
            .iter()
            .map(|s| s.as_str())
            .take(2)
            .collect::<Vec<&str>>(),
    );
    let cave = caves.remove(choice);

    spacer(ctx);

    ctx.term
        .write_line(&fmt_line!(
            ctx.hottext,
            "caves.enter",
            cave = cave_names[choice].as_str()
        ))
        .unwrap();

    // Increment stats
    world.stats.caves += 1;
    world.stats.monsters += cave.monsters.len() as u64;

    let mut xp = 500; // Minimum cave XP

    if cave.monsters.is_empty() {
        ctx.term
            .write_line(&get_line!(ctx.hottext, "combat.no-enemies"))
            .unwrap();
    } else {
        for monster in cave.monsters {
            spacer(ctx);

            ctx.term
                .write_line(&fmt_line!(
                    ctx.hottext,
                    if monster.is_difficult(player.level()) {
                        "combat.encounter-hard"
                    } else {
                        "combat.encounter-easy"
                    },
                    enemy = monster.name().as_str(),
                    enemy_article = monster.article_name().as_str(),
                    enemy_proper = monster.proper_name().as_str()
                ))
                .unwrap();

            // Roll for initiative
            let initiative =
                ((player.level().max(1)) as f64 / (monster.level() * 2) as f64).clamp(0.0, 1.0);
            if ctx.rng.gen_bool(initiative) {
                ctx.term
                    .write_line(
                        fmt_line!(
                            ctx.hottext,
                            "combat.initiative",
                            enemy = monster.name().as_str(),
                            enemy_article = monster.article_name().as_str(),
                            enemy_proper = monster.proper_name().as_str()
                        )
                        .as_str(),
                    )
                    .unwrap();
            } else {
                // Monster drastically outclasses player
                if monster.is_difficult(player.level()) {
                    // Roll to escape
                    if ctx.rng.gen_bool(0.20) {
                        ctx.term
                            .write_line(&fmt_line!(
                                ctx.hottext,
                                "combat.retreat",
                                enemy = monster.name().as_str(),
                                enemy_article = monster.article_name().as_str(),
                                enemy_proper = monster.proper_name().as_str()
                            ))
                            .unwrap();
                        continue;
                    }
                }

                let damage = monster.damage(&mut ctx.rng);
                let applied_damage = player.add_damage(damage, world);
                let damage_str = format!("{} damage", applied_damage.commas())
                    .color(colors::DAMAGE)
                    .to_string();
                ctx.term
                    .write_line(&fmt_line!(
                        ctx.hottext,
                        "combat.attacked",
                        damage = damage_str.as_str(),
                        enemy = monster.name().as_str(),
                        enemy_article = monster.article_name().as_str(),
                        enemy_proper = monster.proper_name().as_str()
                    ))
                    .unwrap();

                // Attempt to heal
                if let Some(potions_used) = player.auto_heal(world) {
                    for potion in potions_used {
                        ctx.term
                            .write_line(&fmt_line!(
                                ctx.hottext,
                                "potion.use",
                                potion = world
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
                    ctx.term
                        .write_line(
                            fmt_line!(
                                ctx.hottext,
                                "combat.player-turn",
                                enemy = monster.name().as_str(),
                                enemy_article = monster.article_name().as_str(),
                                enemy_proper = monster.proper_name().as_str()
                            )
                            .as_str(),
                        )
                        .unwrap();

                    ctx.term
                        .write_line(
                            fmt_line!(
                                ctx.hottext,
                                "combat.survived",
                                enemy = monster.name().as_str(),
                                enemy_article = monster.article_name().as_str(),
                                enemy_proper = monster.proper_name().as_str()
                            )
                            .as_str(),
                        )
                        .unwrap();
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
    ctx: &mut Context,
) -> CaveSurvivedChoice {
    spacer(ctx);

    // Loot
    player.add_xp(reward.xp);
    player.add_gold(reward.gold);

    ctx.term
        .write_line(&get_line!(ctx.hottext, "combat.reward"))
        .unwrap();

    let xp_length = reward.xp.commas().len();
    let show_item = |(name, count): (&str, u32)| {
        let count_str = format!("x{:<width$} -", count.commas(), width = xp_length)
            .color(colors::LOW_PRIORITY)
            .to_string();
        println!("{} {}", count_str, name);
    };
    show_item((
        "xp".color(colors::XP).to_string().as_ref(),
        reward.xp as u32,
    ));
    show_item((
        "gold".color(colors::GOLD).to_string().as_ref(),
        reward.gold as u32,
    ));
    for (item, count) in reward.loot {
        for _ in 0..count {
            player.add_item(&item);
        }
        show_item((
            world
                .get_item(&item)
                .expect("world.items should not have mutated")
                .name()
                .as_ref(),
            count,
        ));
    }
    // TODO: "You leveled up!"

    spacer(ctx);

    let prompt = get_line!(ctx.hottext, "interface.generic-menu");
    let choices: [&str; 2] = [
        &get_line!(ctx.hottext, "interface.next-cave"),
        &get_line!(ctx.hottext, "interface.show-status"),
    ];
    let choice_index = get_choice(ctx, &prompt, &choices);
    match choice_index {
        0 => CaveSurvivedChoice::Continue,
        _ => CaveSurvivedChoice::ShowStatusReport,
    }
}

pub fn show_status(world: &World, player: &Player, ctx: &mut Context) {
    spacer(ctx);

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
    wait_any_key(ctx);
}

pub fn show_inventory(world: &World, player: &Player, ctx: &mut Context) {
    spacer(ctx);

    let mut inventory = player
        .inventory()
        .iter()
        .map(|(item, count)| {
            (
                world
                    .get_item(item)
                    .expect("world.items should not have mutated")
                    .name(),
                *count,
            )
        })
        .collect::<Vec<(String, u32)>>();
    inventory.sort_by_key(|e| e.1);
    inventory.reverse();

    let show_item = |(name, count): (&str, u32)| {
        let count_str = format!(
            "x{:<width$} -",
            count.commas(),
            width = player.gold().commas().len()
        )
        .color(colors::LOW_PRIORITY)
        .to_string();
        println!("{} {}", count_str, name);
    };
    show_item((
        "gold".color(colors::GOLD).to_string().as_ref(),
        player.gold() as u32,
    ));
    inventory
        .into_iter()
        .for_each(|(name, count)| show_item((&name, count)));
}

pub fn show_death_screen(world: &World, player: &Player, ctx: &mut Context) -> GameOverChoice {
    spacer(ctx);

    ctx.term
        .write_line(&get_line!(ctx.hottext, "combat.died"))
        .unwrap();
    ctx.term
        .write_line(&get_line!(ctx.hottext, "combat.game-over"))
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

    let prompt = get_line!(ctx.hottext, "interface.generic-menu");
    let choices: [&str; 4] = [
        &get_line!(ctx.hottext, "interface.retry"),
        &get_line!(ctx.hottext, "interface.view-inventory"),
        &get_line!(ctx.hottext, "interface.leaderboards"),
        &get_line!(ctx.hottext, "interface.quit"),
    ];
    loop {
        spacer(ctx);

        let choice_index = get_choice(ctx, &prompt, &choices);
        match choice_index {
            0 => break GameOverChoice::Retry,
            1 => show_inventory(world, player, ctx),
            2 => todo!(),
            _ => break GameOverChoice::Quit,
        }
    }
}
