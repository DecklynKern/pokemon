use crate::db_enums::*;
use crate::sim::*;
use crate::database::*;
use crate::pokemon::*;
use crate::controller::*;

#[derive(Debug)]
pub struct Side {
    pub team: Vec<Pokemon>,
    pub effects: SideEffects,
    pub active_pokemon: usize
}

impl Side {
    
    pub fn new(pokemon: Vec<Pokemon>) -> Self {
        Self {
            team: pokemon,
            effects: SideEffects::default(),
            active_pokemon: 0
        }
    }

    pub fn get_active(&self) -> &Pokemon {
        &self.team[self.active_pokemon]
    }

    pub fn get_active_mut(&mut self) -> &mut Pokemon {
        &mut self.team[self.active_pokemon]
    }

    pub fn try_apply_status(&mut self, status: NonVolatileStatus) {

        let pokemon = self.get_active_mut();

        if pokemon.non_volatile_status.is_none() {
            pokemon.non_volatile_status = Some(status);
        }
    }
}

pub struct Conditions {
    pub weather: Option<(Weather, u8)>,
    pub terrain: Option<(Terrain, u8)>
}

impl Conditions {

    pub fn default() -> Self {
        Self {
            weather: None,
            terrain: None
        }
    }

    pub fn decriment_counters(&mut self) {

        if let Some((_, turns)) = &mut self.weather {

            if *turns != Weather::PERMANENT {
                   
                *turns -= 1;
                
                if *turns == 0 {
                    self.weather = None;
                }
            }
        }

        if let Some((_, turns)) = &mut self.terrain {

            *turns -= 1;

            if *turns == 0 {
                self.terrain = None;
            }
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

pub struct BattleState {
    pub side1: Side,
    pub side2: Side,
    pub conditions: Conditions
}

impl BattleState {
    pub fn new(side1_pokemon: Vec<Pokemon>, side2_pokemon: Vec<Pokemon>) -> Self {
        Self {
            side1: Side::new(side1_pokemon),
            side2: Side::new(side2_pokemon),
            conditions: Conditions::default()
        }
    }
}

pub enum BattleAction {
    Move(ID),
    Switch(u8),
    Item(ID)
}

pub struct Battle {
    state: BattleState,
    simulator: Simulator,
    controller1: Box<dyn Controller>,
    controller2: Box<dyn Controller>
}

impl Battle {

    pub fn new_battle(data_handler: &'static DataHandler, side1_pokemon: Vec<Pokemon>, side2_pokemon: Vec<Pokemon>, generation: u8) -> Self {
        Self {
            state: BattleState::new(side1_pokemon, side2_pokemon),
            simulator: Simulator::new(data_handler, generation),
            controller1: Box::new(TextController::new(&data_handler)),
            controller2: Box::new(TextController::new(&data_handler)),
            //controller2: Box::new(RandomController::new())
        }
    }

    pub fn battle_ended(&self) -> bool {
        false
    }

    pub fn simulate(&mut self) {

        while !self.battle_ended() {

            let side1_action = self.controller1.get_action(&self.state, true);
            let side2_action = self.controller2.get_action(&self.state, false);
    
            self.simulator.simulate_turn(side1_action, side2_action, &mut self.state);
        
            if self.state.side1.get_active().hp == 0 {
                self.state.side1.active_pokemon = self.controller1.get_switch_in(&self.state, true) as usize;
            }
            
            if self.state.side2.get_active().hp == 0 {
                self.state.side2.active_pokemon = self.controller2.get_switch_in(&self.state, false) as usize;
            }
        }
    }
}