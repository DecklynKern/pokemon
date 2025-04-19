use crate::sim::*;
use crate::database::*;
use crate::pokemon::*;

#[derive(Debug)]
pub struct Side {
    pub team: Vec<Pokemon>,
    pub effects: SideEffects,
    pub active_pokemon: usize,
    pub volatile_status: VolatileStatus
}

impl Side {
    
    pub fn new(pokemon: Vec<Pokemon>) -> Self {
        Self {
            team: pokemon,
            effects: SideEffects::default(),
            active_pokemon: 0,
            volatile_status: VolatileStatus::default()
        }
    }

    pub fn get_active(&self) -> &Pokemon {
        &self.team[self.active_pokemon]
    }

    pub fn get_active_mut(&mut self) -> &mut Pokemon {
        &mut self.team[self.active_pokemon]
    }

    pub fn get_active_stat(&self, stat: Stat) -> u16 {

        let stat_stages = self.volatile_status.stat_stages[stat as usize];

        let mut stat_val = match stat {
            Stat::Evasion | Stat::Accuracy => todo!(),
            Stat::Attack => self.get_active().attack,
            Stat::Defense => self.get_active().defense,
            Stat::SpecialAttack => self.get_active().special_attack,
            Stat::SpecialDefense => self.get_active().special_defense,
            Stat::Speed => self.get_active().speed
        };

        if stat_stages < 0 {
            stat_val *= 2;
            stat_val /= 2 + (-stat_stages as u16);
        }
        else if stat_stages > 0 {
            stat_val *= 2 + stat_stages as u16;
            stat_val /= 2;
        }

        stat_val

    }

    pub fn apply_stat_changes(&mut self, stat: Stat, stages: i8) {
        self.volatile_status.stat_stages[stat as usize] = (self.volatile_status.stat_stages[stat as usize] + stages).clamp(-6, 6);
    }

    pub fn reset_stat_changes(&mut self) {
        self.volatile_status.stat_stages.fill(0);
    }

    pub fn try_apply_status(&mut self, status: NonVolatileStatus) {

        let pokemon = self.get_active_mut();

        if pokemon.status.is_none() {
            pokemon.status = Some(status);
        }
    }
}

pub struct BattleState {
    pub side1: Side,
    pub side2: Side,
    pub weather: Option<(Weather, u8)>,
    pub terrain: Option<(Terrain, u8)>
}

impl BattleState {
    pub fn new(side1_pokemon: Vec<Pokemon>, side2_pokemon: Vec<Pokemon>) -> Self {
        Self {
            side1: Side::new(side1_pokemon),
            side2: Side::new(side2_pokemon),
            weather: None,
            terrain: None
        }
    }

    pub fn is_sunny(&self) -> bool {
        matches!(self.weather, Some((Weather::Sun | Weather::ExtremeSun, _)))
    }

    pub fn is_rain(&self) -> bool {
        matches!(self.weather, Some((Weather::Rain | Weather::HeavyRain, _)))
    }

    pub fn is_weather(&self, weather: Weather) -> bool {
        self.weather.is_some_and(|current| current.0 == weather)
    }

    pub fn is_terrain(&self, terrain: Terrain) -> bool {
        self.terrain.is_some_and(|current| current.0 == terrain)
    }
}

pub enum BattleAction {
    Move(ID),
    Switch(u8),
    Item(ID)
}

pub struct Battle<'a> {
    state: BattleState,
    simulator: Simulator<'a>,
    // controller1: Box<dyn Controller>,
    // controller2: Box<dyn Controller>
}

impl<'a> Battle<'a> {

    pub fn new_battle(data_handler: &'a DataHandler, side1_pokemon: Vec<Pokemon>, side2_pokemon: Vec<Pokemon>, generation: u8) -> Self {
        Self {
            state: BattleState::new(side1_pokemon, side2_pokemon),
            simulator: Simulator::new(data_handler, generation)
        }
    }

    pub fn simulate(&mut self) {
        self.simulator.simulate_turn(todo!(), todo!(), &mut self.state);
    }
}