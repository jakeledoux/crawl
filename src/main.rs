use std::error::Error;

pub mod entities;
mod game;
pub mod interface;

use entities::{player::Player, World};

fn main() -> Result<(), Box<dyn Error>> {
    let mut context = interface::Context::default();
    context
        .hottext
        .load_json("./data/localization/en-us.json")?;

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
        game::show_status(&world, &player, &mut context);
        let result = game::enter_cave(&mut world, &mut player, &mut context);
        if let game::CaveResult::Survived { reward } = result {
            game::show_cave_reward(&mut world, &mut player, reward, &mut context);
        } else {
            break;
        }
    }

    // Game over
    game::show_death_screen(&world, &player, &mut context);

    Ok(())
}
