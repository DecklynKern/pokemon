use crate::db_enums::*;

pub type ID = std::num::NonZeroU16;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Normal = 1,
    Fighting,
    Flying,
    Poison,
    Ground,
    Rock,
    Bug,
    Ghost,
    Steel,
    Fire,
    Water,
    Grass,
    Electric,
    Psychic,
    Ice,
    Dragon,
    Dark,
    Fairy,
    Stellar,
    Unknown
}

impl Type {
    pub fn from_db_id(id: u16) -> Self {

        if id > Self::Stellar as u16 {
            Self::Unknown
        }
        else {
            unsafe {std::mem::transmute(id as u8)}
        }
    }
}

const NUM_TYPES: usize = std::mem::variant_count::<Type>();

pub struct TypeChart([[u8; NUM_TYPES]; NUM_TYPES]);

impl TypeChart {
    pub fn empty() -> Self {
        Self([[100; NUM_TYPES]; NUM_TYPES])
    }

    pub fn set(&mut self, attacker: Type, defender: Type, damage_factor: u8) {
        self.0[attacker as usize][defender as usize] = damage_factor;
    }

    pub fn get(&self, attacker: Type, defender: Type) -> u8 {
        self.0[attacker as usize][defender as usize]
    }
}

pub struct PokemonData {
    pub species_id: ID,
    pub name: Box<str>,
    pub type1: Type,
    pub type2: Option<Type>,
    pub hp: u8,
    pub attack: u8,
    pub defense: u8,
    pub special_attack: u8,
    pub special_defense: u8,
    pub speed: u8,
    pub ability1: Ability,
    pub ability2: Option<Ability>,
    pub hidden_ability: Option<Ability>,
    pub weight: f32,   
}

impl PokemonData {

    pub fn is_type(&self, check_type: Type) -> bool {
        
        if let Some(type2) = self.type2 {
            if type2 == check_type {
                return true;
            }
        }
            
        self.type1 == check_type

    }
}

#[derive(Debug, Clone)]
pub struct Pokemon {
    pub id: ID,
    pub level: u8,
    pub ability: Ability,
    pub hp: u16,
    pub max_hp: u16,
    pub attack: u16,
    pub defense: u16,
    pub special_attack: u16,
    pub special_defense: u16,
    pub speed: u16,
    pub moves: [Option<ID>; 4],
    pub status: Option<NonVolatileStatus>,
    pub held_item: Option<Item>,
    pub gender: Gender,
    pub friendship: u8
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gender {
    Male,
    Female,
    Genderless
}

impl Gender {
    pub fn from_char(chr: char) -> Self {
        match chr.to_ascii_lowercase() {
            'm' => Self::Male,
            'f' => Self::Female,
            _ => Self::Genderless
        }
    }

    pub fn is_same(self, other: Self) -> bool {
        self == other && self != Self::Genderless
    }

    pub fn is_opposite(self, other: Self) -> bool {
        matches!((self, other), (Self::Male, Self::Female) | (Self::Female, Self::Male))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NonVolatileStatus {
    Burn,
    Freeze,
    Paralysis,
    Poison,
    BadlyPoison,
    Sleep
}

#[derive(Debug, Clone, Copy)]
pub enum VolatileStatusEffect {
    AbilityChange(ID),
    AbilitySuppression,
    TypeChange(!),
    Mimic,
    Substitute(u16),
    Transform(!),
    Illusion,
    Bind {health_fraction: u8, turns: u8},
    Curse,
    Nightmare,
    PerishSong(u8),
    Seed,
    Autotomize(u8),
    Identified,
    Minimize,
    TarShot,
    Grounded,
    MagnetRise,
    Telekinesis,
    AquaRing,
    Ingrain,
    LaserFocus(u8),
    Aim,
    Drowsy,
    Charge,
    Stockpile(u8),
    DefenseCurl,
    NoRetreat,
    Octolock,
    Disable(u8),
    Embargo(u8),
    HealBlock(u8),
    Imprison(!),
    Taunt(u8),
    ThroatChop(u8),
    Torment(u8),
    Confusion(u8),
    Infatuation(!),
    GettingPumped,
    GuardSplit(!),
    PowerSplit(!),
    SpeedSwap(!),
    PowerTrick,
    Choiced(u8),
    Encore(u8),
    Rampage(!),
    Rolling(!),
    Uproar(u8),
    Bide(u16, u8),
    Recharge,
    Charging(!),
    SemiInvulernable(!),
    Flinch,
    Endure,
    CenterOfAttention,
    MagicCoat,
    Protect(!)
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub enum Stat {
    Attack,
    Defense,
    SpecialAttack,
    SpecialDefense,
    Speed,
    Evasion,
    Accuracy
}

const NUM_STATS: usize = std::mem::variant_count::<Stat>();

#[derive(Debug)]
pub struct VolatileStatus {
    pub stat_stages: [i8; NUM_STATS],
    pub effects: Vec<VolatileStatusEffect>
}

impl VolatileStatus {
    pub fn default() -> Self {
        Self {
            stat_stages: [0; NUM_STATS],
            effects: Vec::new()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Weather {
    Sun,
    Rain,
    Sandstorm,
    Hail,
    Snow,
    Fog,
    ExtremeSun,
    HeavyRain,
    StrongWind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Terrain {
    Electric,
    Grassy,
    Psychic,
    Misty
}

macro_rules! fields {
    ($type:ty [$start_bit:expr]) => {};
    ($type:ty [$start_bit:expr] 1 $(|$bits:literal)* $read_fn:ident $set_fn:ident $($tok:tt)*) => {

        pub fn $read_fn(&self) -> bool {
            self.0 << ($start_bit) >> (Self::NUM_BITS - 1) != 0
        }

        pub fn $set_fn(&mut self, value: bool) {
            let shift = Self::NUM_BITS - 1 - ($start_bit);
            self.0 &= !(1 << shift);
            self.0 |= (value as $type) << shift;
        }

        fields!($type [$start_bit + 1] $($bits)|* $($tok)*);
    };
    ($type:ty [$start_bit:expr] $size:literal $(|$bits:literal)* $read_fn:ident $set_fn:ident $($tok:tt)*) => {

        pub fn $read_fn(&self) -> u8 {
            (self.0 << ($start_bit) >> (Self::NUM_BITS - $size)) as u8
        }

        pub fn $set_fn(&mut self, value: u8) {
            let shift = Self::NUM_BITS - ($start_bit) - $size;
            self.0 &= !(!(!0 << ($size)) << shift);
            self.0 |= (value as $type) << shift;
        }

        fields!($type [$start_bit + $size] $($bits)|* $($tok)*);
    }
}

macro_rules! bitfield {
    ($name:ident($type:ty); $($tok:tt)+) => {
        
        #[derive(Debug)]
        pub struct $name($type);

        impl $name {
            const NUM_BITS: usize = std::mem::size_of::<$type>() * 8;
            pub fn default() -> Self {
                Self(0)
            }
            fields!($type [0] $($tok)+);
        }
    }
}

// ABBCCDEEEEFFFFGGGGHHHIIIIJJJKKKL
// A - stealth rock
// B - spikes
// C - toxic spikes
// D - sticky web
// E - reflect
// F - light screen
// G - safeguard
// H - mist
// I - aurora veil
// J - tailwind
// K - lucky chant
// L - happy hour
bitfield!(
    SideEffects(u32);
    1|2|2|1|4|4|4|3|4|3|3|1
    get_stealth_rock set_stealth_rock
    get_spikes set_spikes
    get_toxic_spikes set_toxic_spikes
    get_sticky_web set_sticky_web
    get_reflect set_reflect
    get_light_screen set_light_screen
    get_safeguard set_safeguard
    get_mist set_mist
    get_aurora_veil set_aurora_veil
    get_tailwind set_tailwind
    get_lucky_chant set_lucky_chant
    get_happy_hour set_happy_hour
);

impl SideEffects {
    pub fn add_spikes(&mut self) {
        self.set_spikes((self.get_spikes() + 1).min(3));
    }
    pub fn add_toxic_spikes(&mut self) {
        self.set_spikes((self.get_spikes() + 1).min(2));
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveClass {
    Physical,
    Special,
    Status
}

impl MoveClass {
    pub fn from_db_id(id: u8) -> Self {

        match id {
            2 => Self::Physical,
            3 => Self::Special,
            _ => Self::Status
        }
    }
}

// bitfield!(
//     MoveFlags(u32);
//     1|1|1|1|1|1|1|1|1|1|1|1|1|1|1|1|1|1|1|1|1
//     get_contact set_contact
//     get_charge set_charge
//     get_recharge set_recharge
//     get_protect set_protect
//     get_reflectable set_reflectable
//     get_snatch set_snatch
//     get_mirror set_mirror
//     get_punch set_punch
//     get_sound set_sound
//     get_gravity set_gravity
//     get_defrost set_defrost
//     get_distance set_distance
//     get_heal set_heal
//     get_authentic set_authentic
//     get_powder set_powder
//     get_bite set_bite
//     get_pulse set_pulse
//     get_ballistics set_ballistics
//     get_mental set_mental
//     get_non_sky_battle set_non_sky_battle
//     get_dance set_dance
// );

#[derive(Debug)]
pub struct MoveFlags(u32);

impl MoveFlags {
    const NUM_BITS: usize = std::mem::size_of::<u32>() * 8;
    pub fn default() -> Self {
        Self(0)
    }
    pub fn get_contact(&self) -> bool {
        self.0 << (0) >> (Self::NUM_BITS - 1) != 0
    }
    pub fn set_contact(&mut self, value: bool) {
        let shift = Self::NUM_BITS - 1 - (0);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_charge(&self) -> bool {
        self.0 << (0 + 1) >> (Self::NUM_BITS - 1) != 0
    }
    pub fn set_charge(&mut self, value: bool) {
        let shift = Self::NUM_BITS - 1 - (0 + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_recharge(&self) -> bool {
        self.0 << ((0 + 1) + 1) >> (Self::NUM_BITS - 1) != 0
    }
    pub fn set_recharge(&mut self, value: bool) {
        let shift = Self::NUM_BITS - 1 - ((0 + 1) + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_protect(&self) -> bool {
        self.0 << (((0 + 1) + 1) + 1) >> (Self::NUM_BITS - 1) != 0
    }
    pub fn set_protect(&mut self, value: bool) {
        let shift = Self::NUM_BITS - 1 - (((0 + 1) + 1) + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_reflectable(&self) -> bool {
        self.0 << ((((0 + 1) + 1) + 1) + 1) >> (Self::NUM_BITS - 1) != 0
    }
    pub fn set_reflectable(&mut self, value: bool) {
        let shift = Self::NUM_BITS - 1 - ((((0 + 1) + 1) + 1) + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_snatch(&self) -> bool {
        self.0 << (((((0 + 1) + 1) + 1) + 1) + 1) >> (Self::NUM_BITS - 1) != 0
    }
    pub fn set_snatch(&mut self, value: bool) {
        let shift = Self::NUM_BITS - 1 - (((((0 + 1) + 1) + 1) + 1) + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_mirror(&self) -> bool {
        self.0 << ((((((0 + 1) + 1) + 1) + 1) + 1) + 1) >> (Self::NUM_BITS - 1) != 0
    }
    pub fn set_mirror(&mut self, value: bool) {
        let shift = Self::NUM_BITS - 1 - ((((((0 + 1) + 1) + 1) + 1) + 1) + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_punch(&self) -> bool {
        self.0 << (((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) >> (Self::NUM_BITS - 1) != 0
    }
    pub fn set_punch(&mut self, value: bool) {
        let shift = Self::NUM_BITS - 1 - (((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_sound(&self) -> bool {
        self.0 << ((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) >> (Self::NUM_BITS - 1) != 0
    }
    pub fn set_sound(&mut self, value: bool) {
        let shift = Self::NUM_BITS - 1 - ((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_gravity(&self) -> bool {
        self.0 << (((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) >> (Self::NUM_BITS - 1)
            != 0
    }
    pub fn set_gravity(&mut self, value: bool) {
        let shift = Self::NUM_BITS - 1 - (((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_defrost(&self) -> bool {
        self.0 << ((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1)
            >> (Self::NUM_BITS - 1)
            != 0
    }
    pub fn set_defrost(&mut self, value: bool) {
        let shift =
            Self::NUM_BITS - 1 - ((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_distance(&self) -> bool {
        self.0 << (((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1)
            >> (Self::NUM_BITS - 1)
            != 0
    }
    pub fn set_distance(&mut self, value: bool) {
        let shift = Self::NUM_BITS
            - 1
            - (((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_heal(&self) -> bool {
        self.0 << ((((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1)
            >> (Self::NUM_BITS - 1)
            != 0
    }
    pub fn set_heal(&mut self, value: bool) {
        let shift = Self::NUM_BITS
            - 1
            - ((((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_authentic(&self) -> bool {
        self.0 << (((((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1)
            >> (Self::NUM_BITS - 1)
            != 0
    }
    pub fn set_authentic(&mut self, value: bool) {
        let shift = Self::NUM_BITS
            - 1
            - (((((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_powder(&self) -> bool {
        self.0
            << ((((((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1)
            >> (Self::NUM_BITS - 1)
            != 0
    }
    pub fn set_powder(&mut self, value: bool) {
        let shift = Self::NUM_BITS
            - 1
            - ((((((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_bite(&self) -> bool {
        self.0
            << (((((((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1)
                + 1)
                + 1)
            >> (Self::NUM_BITS - 1)
            != 0
    }
    pub fn set_bite(&mut self, value: bool) {
        let shift = Self::NUM_BITS
            - 1
            - (((((((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1)
                + 1)
                + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_pulse(&self) -> bool {
        self.0
            << ((((((((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1)
                + 1)
                + 1)
                + 1)
            >> (Self::NUM_BITS - 1)
            != 0
    }
    pub fn set_pulse(&mut self, value: bool) {
        let shift = Self::NUM_BITS
            - 1
            - ((((((((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1)
                + 1)
                + 1)
                + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_ballistics(&self) -> bool {
        self.0
            << (((((((((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
            >> (Self::NUM_BITS - 1)
            != 0
    }
    pub fn set_ballistics(&mut self, value: bool) {
        let shift = Self::NUM_BITS
            - 1
            - (((((((((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_mental(&self) -> bool {
        self.0
            << ((((((((((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
            >> (Self::NUM_BITS - 1)
            != 0
    }
    pub fn set_mental(&mut self, value: bool) {
        let shift = Self::NUM_BITS
            - 1
            - ((((((((((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_non_sky_battle(&self) -> bool {
        self.0
            << (((((((((((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
            >> (Self::NUM_BITS - 1)
            != 0
    }
    pub fn set_non_sky_battle(&mut self, value: bool) {
        let shift = Self::NUM_BITS
            - 1
            - (((((((((((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
    pub fn get_dance(&self) -> bool {
        self.0
            << ((((((((((((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
            >> (Self::NUM_BITS - 1)
            != 0
    }
    pub fn set_dance(&mut self, value: bool) {
        let shift = Self::NUM_BITS
            - 1
            - ((((((((((((((((((((0 + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1) + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1)
                + 1);
        self.0 &= !(1 << shift);
        self.0 |= (value as u32) << shift;
    }
}

pub struct Move {
    pub id: ID,
    pub class: MoveClass,
    pub move_type: Type,
    pub priority: i8,
    pub power: Option<u8>,
    pub accuracy: Option<u8>,
    pub effect: Option<(MoveEffect, u8)>,
    // pub target: MoveTarget,
    pub flags: MoveFlags
}

impl Move {
    pub fn has_effect(&self, effect: MoveEffect) -> bool {
        self.effect.is_some_and(|(move_effect, _)| move_effect == effect)
    }
}