#![feature(variant_count)]
#![feature(never_type)]
#![feature(tuple_trait)]
#![feature(concat_idents)]

#[macro_use]
mod bitfield;
mod db_enums;
mod pokemon;
mod database;
mod battle;
mod sim;
mod parser;
mod controller;
mod logging;

use database::*;
use battle::*;
use parser::*;

const MY_TEAM: &'static str = "
Volcarona @ Miracle Seed
Level: 65
Careful Nature
Ability: Flame Body
EVs: 20 HP / 20 Def / 252 SpA / 20 SpD / 100 Spe
IVs: 2 HP / 29 Atk / 19 Def / 22 SpA / 19 SpD / 6 Spe
- Quiver Dance
- Flamethrower
- Signal Beam
- Giga Drain";

const IRIS_TEAM: &'static str = "
Hydreigon @ Wise Glasses
Level: 57
Modest Nature
Ability: Levitate
IVs: 30 HP / 30 Atk / 30 Def / 30 SpA / 30 SpD / 30 Spe
- Dragon Pulse
- Fire Blast
- Focus Blast
- Surf

Druddigon @ Life Orb
Level: 57
Naughty Nature
Ability: Sheer Force
IVs: 30 HP / 30 Atk / 30 Def / 30 SpA / 30 SpD / 30 Spe
- Fire Punch
- Focus Blast
- Outrage
- Thunder Punch

Aggron @ Muscle Band
Level: 57
Careful Nature
Ability: Rock Head
IVs: 30 HP / 30 Atk / 30 Def / 30 SpA / 30 SpD / 30 Spe
- Head Smash
- Double-Edge
- Earthquake
- Autotomize

Archeops @ Flying Gem
Level: 57
Brave Nature
Ability: Defeatist
IVs: 30 HP / 30 Atk / 30 Def / 30 SpA / 30 SpD / 30 Spe
- Acrobatics
- Dragon Claw
- Stone Edge
- Endeavor

Lapras @ Wide Lens
Level: 57
Hasty Nature
Ability: Water Absorb
IVs: 30 HP / 30 Atk / 30 Def / 30 SpA / 30 SpD / 30 Spe
- Hydro Pump
- Blizzard
- Thunder
- Sing

Haxorus @ Focus Sash
Level: 59
Docile Nature
Ability: Mold Breaker
IVs: 30 HP / 30 Atk / 30 Def / 30 SpA / 30 SpD / 30 Spe
- Earthquake
- Outrage
- X-Scissor
- Dragon Dance";

static mut DATA_HANDLER: Option<DataHandler> = None;

fn get_data_handler() -> &'static DataHandler {
    unsafe {
        DATA_HANDLER.as_ref().unwrap()
    }
}

fn main() -> std::io::Result<()> {

    unsafe {
        DATA_HANDLER = Some(DataHandler::new()?);
    }

    =

    let my_team = parse_showdown_team(MY_TEAM, get_data_handler());
    let iris_team = parse_showdown_team(IRIS_TEAM, get_data_handler());

    let mut battle = Battle::new_battle(get_data_handler(), my_team, iris_team, 5);
    battle.simulate();

    Ok(())

}
