use crate::battle::*;

pub trait Controller {
    fn get_action(&self, state: &BattleState) -> BattleAction;
    fn get_switch_in(&self, state: &BattleState) -> u8;
}

pub struct TextController {
}

impl Controller for TextController {
    fn get_action(&self, state: &BattleState) -> BattleAction {
        todo!()
    }
    fn get_switch_in(&self, state: &BattleState) -> u8 {
        todo!()
    }
}

pub struct RandomController {
}

impl Controller for RandomController {
    fn get_action(&self, state: &BattleState) -> BattleAction {
        todo!()
    }
    fn get_switch_in(&self, state: &BattleState) -> u8 {
        todo!()
    }
}

impl RandomController {
    pub fn new() -> Self {
        Self {}
    }
}