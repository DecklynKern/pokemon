#![feature(variant_count)]
#![feature(never_type)]
#![feature(tuple_trait)]
#![feature(concat_idents)]

mod db_enums;
mod pokemon;
mod database;
mod battle;
mod sim;
mod parser;
mod controller;

use database::*;
use battle::*;
use parser::*;

const TEAM: &'static str = "Gallade-Mega (M) @ Galladite  
Ability: Justified  
EVs: 224 HP / 32 Atk / 252 Spe  
Jolly Nature  
- Swords Dance  
- Shadow Sneak  
- Psycho Cut  
- Drain Punch  

Serperior @ Chesto Berry  
Ability: Contrary  
EVs: 200 HP / 44 Def / 12 SpA / 252 Spe  
Timid Nature  
IVs: 1 Atk / 30 Def / 30 SpD / 30 Spe  
- Rest  
- Glare  
- Hidden Power [Rock]  
- Leaf Storm  

Crawdaunt @ Life Orb  
Ability: Adaptability  
EVs: 252 Atk / 4 SpD / 252 Spe  
Jolly Nature  
- Swords Dance  
- Aqua Jet  
- Crabhammer  
- Knock Off  

Excadrill @ Leftovers  
Ability: Mold Breaker  
EVs: 128 Atk / 176 SpD / 204 Spe  
Adamant Nature  
- Rapid Spin  
- Substitute  
- Iron Head  
- Earthquake  

Zapdos @ Leftovers  
Ability: Static  
EVs: 248 HP / 236 Def / 24 Spe  
Bold Nature  
IVs: 0 Atk / 30 Def  
- Roost  
- Hidden Power [Ice]  
- Heat Wave  
- Volt Switch  

Clefable @ Leftovers  
Ability: Magic Guard  
EVs: 248 HP / 200 Def / 56 SpD / 4 Spe  
Calm Nature  
IVs: 0 Atk  
- Stealth Rock  
- Soft-Boiled  
- Healing Wish  
- Moonblast 
";

fn main() -> std::io::Result<()> {

    let data_handler = DataHandler::new()?;

    let mut team = Vec::new();
    let mut mon_lines = Vec::new();

    for line in TEAM.split('\n') {
        
        if line.is_empty() {
            team.push(parse_showdown(&mon_lines, &data_handler));
            mon_lines.clear();
        }
        else {
            mon_lines.push(line);
        }
    }

    let team2 = team.clone();

    let mut battle = Battle::new_battle(&data_handler, team, team2, 5);
    battle.simulate();

    Ok(())

}
