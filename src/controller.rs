use crate::battle::*;

pub trait Controller {
    fn get_action(state: &BattleState) -> BattleAction;
    fn get_switch_in(state: &BattleState) -> u8;
}

pub struct TextController {
}

impl Controller for TextController {
    fn get_action(state: &BattleState) -> BattleAction {
        todo!()
    }
    fn get_switch_in(state: &BattleState) -> u8 {
        todo!()
    }
}

pub struct RandomController {
}

impl Controller for RandomController {
    fn get_action(state: &BattleState) -> BattleAction {
        todo!()
    }
    fn get_switch_in(state: &BattleState) -> u8 {
        todo!()
    }
}