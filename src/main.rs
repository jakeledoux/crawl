use std::error::Error;

use clap::Clap;

pub mod colors {
    use colored::*;

    pub const LOW_PRIORITY: Color = Color::TrueColor {
        r: 170,
        g: 170,
        b: 170,
    };
    pub const INPUT: Color = LOW_PRIORITY;

    pub const XP: Color = Color::Cyan;
    pub const GOLD: Color = Color::Yellow;
    pub const DAMAGE: Color = Color::BrightRed;

    pub const ITEM: Color = Color::BrightBlue;
    pub const MONSTER: Color = Color::BrightYellow;
}
pub mod entities;
pub mod game;
pub mod interface;

use entities::{player::Player, World};

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Jake Ledoux <me@jakeledoux.com>")]
struct Opts {
    #[clap(short, long, default_value = "en-US", possible_values = &["en-US", "en-PR"])]
    locale: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();

    let mut ctx = interface::Context::default();
    ctx.hottext
        .load_json(format!("./data/localization/{}.json", opts.locale))
        .expect("No localization file exists for given locale.");

    let mut world = World::new()
        .with_load_monsters("./data/monsters/generic.json")?
        .with_load_monsters("./data/monsters/unique.json")?
        .with_load_items("./data/items/armor.json")?
        .with_load_items("./data/items/shields.json")?
        .with_load_items("./data/items/collectibles.json")?
        .with_load_items("./data/items/potions.json")?
        .with_load_items("./data/items/weapons.json")?;

    loop {
        // Reset world and player for a new game
        world.reset();
        let mut player = Player::default();

        loop {
            let result = game::enter_cave(&mut world, &mut player, &mut ctx);
            if let game::CaveResult::Survived { reward } = result {
                if game::show_cave_reward(&mut world, &mut player, reward, &mut ctx)
                    .is_show_status_report()
                {
                    game::show_status(&world, &player, &mut ctx);
                }
            } else {
                break;
            }
        }

        // Game over
        if game::show_death_screen(&world, &player, &mut ctx).is_quit() {
            break;
        }
    }

    Ok(())
}
