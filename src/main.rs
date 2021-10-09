use std::error::Error;

use console::Term;

pub mod entities;
mod game;
pub mod interface;

use entities::{player::Player, World};

fn main() -> Result<(), Box<dyn Error>> {
    // Load templating resources

    let mut term = Term::stdout();

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
        game::show_status(&world, &player, &mut term);
        let result = game::enter_cave(&mut world, &mut player, &mut rng, &mut term);
        if let game::CaveResult::Survived { reward } = result {
            game::show_cave_reward(&mut world, &mut player, reward, &mut term);
        } else {
            break;
        }
    }

    // Game over
    game::show_death_screen(&world, &player, &mut term);

    Ok(())
}
