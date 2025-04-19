use crate::pokemon::ID;

const fn id(num: u16) -> ID {
    ID::new(num).unwrap()
} 

pub const STAT_HP_ID: u8 = 1;
pub const STAT_ATTACK_ID: u8 = 2;
pub const STAT_DEFENSE_ID: u8 = 3;
pub const STAT_SPECIAL_ATTACK_ID: u8 = 4;
pub const STAT_SPECIAL_DEFENSE_ID: u8 = 5;
pub const STAT_SPEED_ID: u8 = 6;

pub const POKEMON_SHEDINJA_ID: ID = id(292);

pub const MOVE_POUND_ID: ID = id(1);