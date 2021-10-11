use std::error::Error;

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

fn main() -> Result<(), Box<dyn Error>> {
    let mut ctx = interface::Context::default();
    ctx.hottext.load_json("./data/localization/en-us.json")?;

    let mut world = World::new()
        .with_load_monsters("./data/monsters/generic.json")?
        .with_load_monsters("./data/monsters/unique.json")?
        .with_load_items("./data/items/armor.json")?
        .with_load_items("./data/items/shields.json")?
        .with_load_items("./data/items/collectibles.json")?
        .with_load_items("./data/items/potions.json")?
        .with_load_items("./data/items/weapons.json")?;
    let mut player = Player::default();

    loop {
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
