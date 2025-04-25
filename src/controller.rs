use crate::battle::*;
use crate::pokemon::*;
use crate::database::DataHandler;

pub trait Controller {
    fn get_action(&self, state: &BattleState, is_side1: bool) -> BattleAction;
    fn get_switch_in(&self, state: &BattleState, is_side1: bool) -> u8;
}

pub struct TextController {
    data_handler: &'static DataHandler
}

impl Controller for TextController {

    fn get_action(&self, state: &BattleState, is_side1: bool) -> BattleAction {

        let mut my_side = &state.side1;
        let mut other_side = &state.side2;

        if !is_side1 {
            std::mem::swap(&mut my_side, &mut other_side);
        }

        let my_mon = my_side.get_active();
        let other_mon = other_side.get_active();

        println!();

        Self::write_end_line();

        let mut top_line_chars = Self::BOX_WIDTH - 4;
        
        let my_mon_section = my_mon.name.to_string() + " Lv. " + &format!("{}", my_mon.level);
        let other_mon_section = other_mon.name.to_string() + " Lv. " + &format!("{}", other_mon.level);

        top_line_chars -= my_mon_section.chars().count();
        top_line_chars -= other_mon_section.chars().count();

        let top_line_spaces = " ".repeat(top_line_chars);

        println!("| {my_mon_section}{top_line_spaces}{other_mon_section} |");

        let health_line_chars = Self::BOX_WIDTH - Self::HEALTH_BAR_WIDTH * 2 - 4;
        println!("| {}{}{} |", Self::get_health_bar(my_mon), " ".repeat(health_line_chars), Self::get_health_bar(other_mon));

        let pokemon_line_chars = Self::BOX_WIDTH - 4 - 6 * 2;
        println!("| {}{}{} |", Self::get_pokemon_display(my_side), " ".repeat(pokemon_line_chars), Self::get_pokemon_display(other_side));

        Self::write_end_line();

        for (idx, my_move_option) in my_mon.moves.iter().enumerate() {

            let Some(my_move) = my_move_option
            else {
                continue;
            };

            let move_data = self.data_handler.get_move(*my_move);

            println!("[{}] {}", idx + 1, move_data.name);
        
        }

        println!("[S]witch");

        let stdin = std::io::stdin();
        let mut input = String::new();

        loop {

            println!("What will you do?");
            stdin.read_line(&mut input).unwrap();

            let Some(char) = input.chars().next()
            else {
                continue;
            };

            match char.to_ascii_lowercase() {
                '1' => return BattleAction::Move(my_mon.moves[0].unwrap()),
                '2' => return BattleAction::Move(my_mon.moves[1].unwrap()),
                '3' => return BattleAction::Move(my_mon.moves[2].unwrap()),
                '4' => return BattleAction::Move(my_mon.moves[3].unwrap()),
                's' => return BattleAction::Switch(self.get_switch_in(state, is_side1)),
                _ => {}
            }
        }
    }

    fn get_switch_in(&self, state: &BattleState, is_side1: bool) -> u8 {

        let mut my_side = &state.side1;
        let mut other_side = &state.side2;

        if !is_side1 {
            std::mem::swap(&mut my_side, &mut other_side);
        }

        let stdin = std::io::stdin();
        let mut input = String::new();

        for (idx, mon) in my_side.team.iter().enumerate() {

            if mon.hp == 0 {
                continue;
            }

            let mon_data = self.data_handler.get_pokemon_data(mon.id);
            println!("[{}] {}", idx + 1, mon_data.name);
        }

        loop {

            println!("Who will you switch in?");
            stdin.read_line(&mut input).unwrap();

            let Some(num) = input.chars().next().map(|char| char as u8 - '1' as u8)
            else {
                continue;
            };

            if num >= 6 {
                continue;
            }

            return num;

        }
    }
}

impl TextController {

    const BOX_WIDTH: usize = 50;
    const HEALTH_BAR_WIDTH: usize = 15;

    pub fn new(data_handler: &'static DataHandler) -> Self {
        Self {
            data_handler
        }
    }

    fn write_end_line() {
        println!("+{}+", "-".repeat(Self::BOX_WIDTH - 2));
    }

    fn get_health_bar(mon: &Pokemon) -> String {
        let chars = (mon.hp as usize) * Self::HEALTH_BAR_WIDTH / (mon.max_hp as usize);
        format!("{}{}", "#".repeat(chars), ".".repeat(Self::HEALTH_BAR_WIDTH - chars))
    }

    fn get_pokemon_display(side: &Side) -> String {

        let mut display = String::new();

        for idx in 0..6 {
            display.push(if let Some(mon) = side.team.get(idx) {
                if mon.hp > 0 {
                    '⬤'
                }
                else {
                    '◯'
                }
            }
            else {
                ' '
            });
        }

        display

    }
}

pub struct RandomController {
}

impl Controller for RandomController {
    fn get_action(&self, state: &BattleState, is_side1: bool) -> BattleAction {
        todo!()
    }
    fn get_switch_in(&self, state: &BattleState, is_side1: bool) -> u8 {
        todo!()
    }
}

impl RandomController {
    pub fn new() -> Self {
        Self {}
    }
}

bitfield!(
    Gen5AIFlags(u16);
    1|1|1|1|1|1|1|1|1|1|1|1|1|1
    get_script0 set_script0
    get_script1 set_script1
    get_script2 set_script2
    get_script3 set_script3
    get_script4 set_script4
    get_script5 set_script5
    get_script6 set_script6
    get_script7 set_script7
    get_script8 set_script8
    get_script9 set_script9
    get_script10 set_script10
    get_script11 set_script11
    get_script12 set_script12
    get_script13 set_script13
);

pub struct Gen5AI {
    data_handler: &'static DataHandler,
    flags: Gen5AIFlags
}

impl Controller for Gen5AI {
    
    fn get_action(&self, state: &BattleState, is_side1: bool) -> BattleAction {

        let mut my_side = &state.side1;
        let mut other_side = &state.side2;

        let my_mon = my_side.get_active();
        let other_mon = other_side.get_active();

        if !is_side1 {
            std::mem::swap(&mut my_side, &mut other_side);
        }

        let move_values = (0..4).map(|idx| {

            let Some(my_move) = my_mon.moves[idx]
            else {
                return -100;
            };

            let move_data = self.data_handler.get_move(my_move);

            let mut score = 100;

            // effectiveness
            if self.flags.get_script0() {
                
                // not ohko
                if true {
                    
                }
            }
    
            // evaluate
            if self.flags.get_script1() {
                
            }
    
            //expert
            if self.flags.get_script2() {
    
            }
    
            // status
            if self.flags.get_script3() {
                
            }
    
            // n final battle
            if self.flags.get_script4() {
                
            }
    
            if self.flags.get_script5() {
                
            }
            
            if self.flags.get_script6() {
                
            }
    
            if self.flags.get_script7() {
                
            }
    
            if self.flags.get_script8() {
                
            }
    
            if self.flags.get_script9() {
                
            }
    
            if self.flags.get_script10() {
            }
    
            if self.flags.get_script11() {
                // run
            }
    
            if self.flags.get_script12() {
                // ?
            }
    
            if self.flags.get_script13() {
                // ?
            }

            score

        }).collect::<Box<[_]>>();

        let mut max_moves = Vec::new();
        let max_score = move_values.iter().max().unwrap();

        for (idx, value) in move_values.iter().enumerate() {
            if value == max_score {
                max_moves.push(idx);
            }
        }

        BattleAction::Move(my_mon.moves[max_moves[rand::random_range(0..max_moves.len())]].unwrap())

    }
    fn get_switch_in(&self, state: &BattleState, is_side1: bool) -> u8 {

        let mut my_side = &state.side1;
        let mut other_side = &state.side2;

        if !is_side1 {
            std::mem::swap(&mut my_side, &mut other_side);
        }

        let mut available_mons = Vec::new();

        for (idx, mon) in my_side.team.iter().enumerate() {

            if idx == my_side.active_pokemon || mon.hp == 0 {
                continue;
            }

            let mut max_power = 0;

            let opponent_mon_data = self.data_handler.get_pokemon_data(other_side.get_active().id);
            
            for mon_move_option in mon.moves {
                
                let Some(mon_move) = mon_move_option
                else {
                    continue;
                };

                let move_data = self.data_handler.get_move(mon_move);

                if move_data.class == MoveClass::Status {
                    continue;
                }

                let mut power = move_data.power.unwrap_or(60) as u16;

                power *= self.data_handler.type_chart.get(move_data.move_type, opponent_mon_data.type1) as u16;
                power /= 100;

                if let Some(type2) = opponent_mon_data.type2 {
                    power *= self.data_handler.type_chart.get(move_data.move_type, type2) as u16;
                    power /= 100;
                }

                max_power = max_power.max(power);

            }

            available_mons.push((idx, max_power));

        }

        let max_power =  available_mons.iter().map(|(_, power)| power).max().unwrap();

        for (idx, power) in &available_mons {
            if power == max_power {
                return *idx as u8;
            }
        }

        return 0;

    }
}

impl Gen5AI {
    pub fn new(data_handler: &'static DataHandler) -> Self {
        Self {
            data_handler,
            flags: Gen5AIFlags(0)
        }
    }
}