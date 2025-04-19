use std::collections::HashMap;

use crate::db_enums::*;
use crate::pokemon::*;

struct CSVDatabase {
    rows: Box<[Box<[String]>]>,
    column_indexes: HashMap<String, usize>
}

impl CSVDatabase {

    fn parse_line(line: &str) -> Box<[String]> {
        line.split(',').map(String::from).collect()
    }

    fn load(path: &str) -> std::io::Result<Self> {

        let file_data = std::fs::read_to_string(path)?;
        let mut lines = file_data.split("\r\n");

        let columns = Self::parse_line(lines.next().unwrap());
        let mut rows: Vec<_> = lines.map(Self::parse_line).collect();

        if rows.last().unwrap()[0].is_empty() {
            rows.pop();
        }

        let mut column_indexes = HashMap::new();

        for (idx, column) in columns.iter().enumerate() {
            column_indexes.insert(column.clone(), idx);
        }

        Ok(Self {
            rows: rows.into_boxed_slice(),
            column_indexes
        })
    }

    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn row_iter(&self) -> std::ops::Range<usize> {
        0..self.row_count()
    }

    fn get_cell<T: std::str::FromStr>(&self, row: usize, col: &str) -> T {
        self.rows[row][self.column_indexes[col]].parse::<T>().map_err(|_|()).unwrap()
    }
}

pub struct NameTable(HashMap<String, ID>);

impl NameTable {

    fn load(path: &str, id_col_name: &str) -> std::io::Result<Self> {
        
        let database = CSVDatabase::load(path)?;
        
        let mut name_table = HashMap::new();

        for row in 0..database.row_count() {
            
            // add language restriction

            let id = database.get_cell(row, id_col_name);
            let name = database.get_cell::<String>(row, "name").to_lowercase();

            name_table.insert(name, id);

        }

        Ok(Self(name_table))

    }

    fn load_unique(path: &str, id_col_name: &str, name_col_name: &str) -> std::io::Result<Self> {
        
        let database = CSVDatabase::load(path)?;
        
        let mut name_table = HashMap::new();

        for row in 0..database.row_count() {
            
            // add language restriction

            let id = database.get_cell(row, id_col_name);
            let name = database.get_cell::<String>(row, name_col_name).to_lowercase();

            name_table.insert(name, id);

        }

        Ok(Self(name_table))

    }

    pub fn lookup_id(&self, name: &str) -> Option<ID> {
        self.0.get(&name.to_lowercase()).cloned()
    }
}

pub struct DataHandler {
    pokemon: HashMap<ID, PokemonData>,
    moves: HashMap<ID, Move>,
    pub form_name_table: NameTable,
    pub ability_name_table: NameTable,
    pub item_name_table: NameTable,
    pub move_name_table: NameTable,
    pub nature_name_table: NameTable,
    pub type_chart: TypeChart,
    nature_chart: [(u8, u8); 25]
}

impl DataHandler {

    pub fn new() -> std::io::Result<Self> {
    
        let pokemon = CSVDatabase::load("data/pokemon.csv")?;
        let pokemon_types = CSVDatabase::load("data/pokemon_types.csv")?;
        let pokemon_stats = CSVDatabase::load("data/pokemon_stats.csv")?;
        let pokemon_abilities = CSVDatabase::load("data/pokemon_abilities.csv")?;
        let moves = CSVDatabase::load("data/moves.csv")?;
        let move_flag_map = CSVDatabase::load("data/move_flag_map.csv")?;
        let type_efficacy = CSVDatabase::load("data/type_efficacy.csv")?;
        let natures = CSVDatabase::load("data/natures.csv")?;

        let mut pokemon_data_table = HashMap::new();
        
        for row in pokemon.row_iter() {

            let id = pokemon.get_cell(row, "id");
            let identifier: Box<str> = pokemon.get_cell::<String>(row, "identifier").into_boxed_str();
            let species_id = pokemon.get_cell(row, "species_id");
            let weight = pokemon.get_cell(row, "weight");

            pokemon_data_table.insert(id, PokemonData {
                species_id,
                name: identifier,
                type1: Type::Normal,
                type2: None,
                hp: 0,
                attack: 0,
                defense: 0,
                special_attack: 0,
                special_defense: 0,
                speed: 0,
                ability1: Ability::None,
                ability2: None,
                hidden_ability: None,
                weight
            });
        }

        for row in pokemon_stats.row_iter() {
            
            let pokemon_id = pokemon_stats.get_cell(row, "pokemon_id");
            let stat_id = pokemon_stats.get_cell(row, "stat_id");
            let base_stat = pokemon_stats.get_cell(row, "base_stat");

            let pokemon_data = pokemon_data_table.get_mut(&pokemon_id).unwrap();

            let stat = match stat_id {
                STAT_HP => &mut pokemon_data.hp,
                STAT_ATTACK => &mut pokemon_data.attack,
                STAT_DEFENSE => &mut pokemon_data.defense,
                STAT_SPECIAL_ATTACK => &mut pokemon_data.special_attack,
                STAT_SPECIAL_DEFENSE => &mut pokemon_data.special_defense,
                STAT_SPEED => &mut pokemon_data.speed,
                _ => continue
            };

            *stat = base_stat;

        }

        for row in pokemon_types.row_iter() {

            let pokemon_id = pokemon_types.get_cell(row, "pokemon_id");
            let pokemon_type = Type::from_db_id(pokemon_types.get_cell(row, "type_id"));
            let slot: String = pokemon_types.get_cell(row, "slot");

            let pokemon_data = pokemon_data_table.get_mut(&pokemon_id).unwrap();

            if slot == "1" {
                pokemon_data.type1 = pokemon_type;
            }
            else {
                pokemon_data.type2 = Some(pokemon_type);
            }
        }

        for row in pokemon_abilities.row_iter() {

            let pokemon_id = pokemon_abilities.get_cell(row, "pokemon_id");
            let ability_id = Ability::from_db_id(pokemon_abilities.get_cell(row, "ability_id"));
            let slot: String = pokemon_abilities.get_cell(row, "slot");

            let pokemon_data = pokemon_data_table.get_mut(&pokemon_id).unwrap();

            match slot.as_str() {
                "1" => pokemon_data.ability1 = ability_id,
                "2" => pokemon_data.ability2 = Some(ability_id),
                "3" => pokemon_data.hidden_ability = Some(ability_id),
                _ => {}
            }
        }

        let mut move_table = HashMap::new();

        for row in moves.row_iter() {

            let id = moves.get_cell(row, "id");
            let move_type = Type::from_db_id(moves.get_cell(row, "type_id"));
            let class = MoveClass::from_db_id(moves.get_cell(row, "damage_class_id"));
            let priority = moves.get_cell(row, "priority");
            let power = moves.get_cell::<String>(row, "power").parse::<u8>().ok();
            let accuracy = moves.get_cell::<String>(row, "accuracy").parse::<u8>().ok();

            move_table.insert(id, Move {
                id,
                class,
                move_type,
                priority,
                power,
                accuracy,
                effect: None, // todo
                flags: MoveFlags::default()
            });
        }

        for row in move_flag_map.row_iter() {

            let move_id = move_flag_map.get_cell(row, "move_id");
            let move_flag_id = move_flag_map.get_cell(row, "move_flag_id");

            let flags = &mut move_table.get_mut(&move_id).unwrap().flags;

            // maybe add enum idk
            match move_flag_id {
                1 => flags.set_contact(true),
                2 => flags.set_charge(true),
                3 => flags.set_recharge(true),
                4 => flags.set_protect(true),
                5 => flags.set_reflectable(true),
                6 => flags.set_snatch(true),
                7 => flags.set_mirror(true),
                8 => flags.set_punch(true),
                9 => flags.set_sound(true),
                10 => flags.set_gravity(true),
                11 => flags.set_defrost(true),
                12 => flags.set_distance(true),
                13 => flags.set_heal(true),
                14 => flags.set_authentic(true),
                15 => flags.set_powder(true),
                16 => flags.set_bite(true),
                17 => flags.set_pulse(true),
                18 => flags.set_ballistics(true),
                19 => flags.set_mental(true),
                20 => flags.set_non_sky_battle(true),
                21 => flags.set_dance(true),
                _ => unreachable!()
            }
        }

        let mut type_chart = TypeChart::empty();

        for row in type_efficacy.row_iter() {

            let attacker = Type::from_db_id(type_efficacy.get_cell(row, "damage_type_id"));
            let defender = Type::from_db_id(type_efficacy.get_cell(row, "target_type_id"));
            let damage_factor = type_efficacy.get_cell(row, "damage_factor");

            type_chart.set(attacker, defender, damage_factor);

        }

        let mut nature_chart = [(0, 0); 25];

        for row in natures.row_iter() {

            let id: usize = natures.get_cell(row, "id");
            let increased_stat = natures.get_cell(row, "increased_stat_id");
            let decreased_stat = natures.get_cell(row, "decreased_stat_id");

            nature_chart[id - 1] = (increased_stat, decreased_stat);

        }

        let form_name_table = NameTable::load_unique("data/pokemon_forms.csv", "pokemon_id", "identifier")?;
        let ability_name_table = NameTable::load("data/ability_names.csv", "ability_id")?;
        let item_name_table = NameTable::load("data/item_names.csv", "item_id")?;
        let move_name_table = NameTable::load("data/move_names.csv", "move_id")?;
        let nature_name_table = NameTable::load_unique("data/natures.csv", "id", "identifier")?;

        Ok(Self {
            pokemon: pokemon_data_table,
            moves: move_table,
            form_name_table,
            ability_name_table,
            item_name_table,
            move_name_table,
            nature_name_table,
            type_chart,
            nature_chart
        })
    }

    pub fn get_pokemon_data(&self, id: ID) -> &PokemonData {
        &self.pokemon[&id]
    }

    pub fn get_move(&self, id: ID) -> &Move {
        &self.moves[&id]
    }

    pub fn get_nature_changed_stats(&self, id: ID) -> (u8, u8) {
        self.nature_chart[id.get() as usize - 1]
    }
}