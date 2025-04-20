use crate::db_enums::*;
use crate::pokemon::*;
use crate::database::*;

fn parse_stats_line(mut line: &str, default: u8) -> [u8; 6] {

    let mut stats = [default; 6];

    line = line.trim();
    if line.is_empty() {
        return stats;
    }

    for stat in line.split('/') {

        let (value, name) = stat.trim().split_once(' ').unwrap();
    
        let idx = match name.trim() {
            "HP" => 0,
            "Atk" => 1,
            "Def" => 2,
            "SpA" => 3,
            "SpD" => 4,
            "Spe" => 5,
            _ => panic!()
        };

        stats[idx] = value.parse().unwrap();

    }

    stats

}

fn calc_hp(base: u8, iv: u8, ev: u8, level: u8) -> u16 {
    let level_u16 = level as u16;
    (2 * (base as u16) + (iv as u16) + ((ev as u16) / 4)) * level_u16 / 100 + level_u16 + 10
}

fn calc_stat(base: u8, iv: u8, ev: u8, level: u8, nature: u8) -> u16  {
    ((2 * (base as u16) + (iv as u16) + ((ev as u16) / 4)) * (level as u16) / 100 + 5) * (nature as u16) / 10
}

pub fn parse_showdown(showdown_export_lines: &[&str], data_handler: &DataHandler) -> Pokemon {

    let mut lines_iter = showdown_export_lines.iter();
    let mut line1 = *lines_iter.next().unwrap();

    let mut gender = Gender::Male;
    let mut held_item = None;

    if let Some((display_name, held_item_name)) = line1.split_once('@') {
        let id = data_handler.item_name_table.lookup_id(held_item_name.trim()).unwrap();
        held_item = Some(Item::from_db_id(id));
        line1 = display_name;
    }

    if let Some((display_name, gender_display)) = line1.split_once('(') {
        gender = Gender::from_char(gender_display.chars().next().unwrap());
        line1 = display_name;
    }

    let form_name = line1.trim();

    let form_id = data_handler.form_name_table.lookup_id(form_name).unwrap();
    let pokemon_data = data_handler.get_pokemon_data(form_id);

    let mut moves = [None; 4];
    let mut move_count = 0;

    let mut level = 100;
    let mut ability = Ability::None;
    let mut ivs = [0; 6];
    let mut evs = [0; 6];
    let mut nature_id = ID::new(1).unwrap();
    let mut friendship = 255;

    for line in lines_iter {

        if line.starts_with("Ability:") {
            let id = data_handler.ability_name_table.lookup_id(line.trim_start_matches("Ability:").trim()).unwrap();
            ability = Ability::from_db_id(id);
        }
        else if line.starts_with("Level:") {
            level = line.trim_start_matches("Level:").trim().parse().unwrap()
        }
        else if line.starts_with("Shiny:") {
        }
        else if line.starts_with("Happiness:") {
            friendship = line.trim_start_matches("Happiness:").trim().parse().unwrap()
        }
        else if line.starts_with("EVs:") {
            evs = parse_stats_line(line.trim_start_matches("EVs:").trim(), 0);
        }
        else if line.starts_with("IVs:") {
            ivs = parse_stats_line(line.trim_start_matches("IVs:").trim(), 31);
        }
        else if line.starts_with('-') {

            let mut move_name = line.trim_start_matches('-');
        
            if let Some((name, _)) = move_name.split_once('[') {
                move_name = name;
            }
        
            moves[move_count] = data_handler.move_name_table.lookup_id(move_name.trim());
            move_count += 1;
        
        }
        else {
            let nature_display = line.split_once(" ").unwrap().0;
            nature_id = data_handler.nature_name_table.lookup_id(nature_display).unwrap();
        }
    }

    let mut nature_modifiers = [10; 5];
    let (increased_stat, decreased_stat) = data_handler.get_nature_changed_stats(nature_id);
    nature_modifiers[increased_stat as usize - 2] += 1;
    nature_modifiers[decreased_stat as usize - 2] -= 1;

    let hp = if form_id == POKEMON_SHEDINJA {
        1
    }
    else {
        calc_hp(pokemon_data.hp, ivs[0], evs[0], level)
    };

    let attack              = calc_stat(pokemon_data.attack,             ivs[1], evs[1], level, nature_modifiers[0]);
    let defense             = calc_stat(pokemon_data.defense,            ivs[2], evs[2], level, nature_modifiers[1]);
    let special_attack      = calc_stat(pokemon_data.special_attack,     ivs[3], evs[3], level, nature_modifiers[2]);
    let special_defense     = calc_stat(pokemon_data.special_defense,    ivs[4], evs[4], level, nature_modifiers[3]);
    let speed               = calc_stat(pokemon_data.speed,              ivs[5], evs[5], level, nature_modifiers[4]);
    
    Pokemon {
        id: form_id,
        level,
        ability,
        item: held_item,
        max_hp: hp,
        hp,
        attack,
        defense,
        special_attack,
        special_defense,
        speed,
        moves,
        non_volatile_status: None,
        volatile_status: VolatileStatus::default(),
        gender,
        friendship
    }
}