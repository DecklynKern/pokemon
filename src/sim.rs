use crate::db_enums::*;
use crate::database::*;
use crate::log;
use crate::pokemon::*;
use crate::battle::*;

macro_rules! find_effect {
    ($mon:expr, $effect:pat => $on_find:block) => {
        for effect in &$mon.volatile_status.effects {
            if let $effect = effect {
                $on_find
            }
        }
    };
}

pub struct HitProperties {
    crit: bool,
    roll_percent: u8,
}

pub struct Simulator {
    data_handler: &'static DataHandler,
    pub generation: u8
}

impl Simulator {

    pub fn new(data_handler: &'static DataHandler, generation: u8) -> Self {
        Self {
            data_handler,
            generation
        }
    }

    fn get_ability(&self, pokemon: &Pokemon, conditions: &Conditions) -> Ability {
    
        find_effect!(pokemon, VolatileStatusEffect::AbilitySuppression => {
            return Ability::None;
        });

        find_effect!(pokemon, VolatileStatusEffect::AbilityChange(ability) => {
            return *ability;
        });
    
        pokemon.ability
    
    }

    fn pokemon_has_type(&self, pokemon: &Pokemon, check_type: Type) -> bool {
        todo!()
    }

    // protosynthesis/quark drive??????

    fn get_attack_stat(&self, pokemon: &Pokemon, conditions: &Conditions) -> u16 {

        let mut attack = pokemon.get_stat(Stat::Attack);

        let ability = self.get_ability(pokemon, conditions);

        match ability {
            Ability::FlowerGift => if conditions.is_sunny() {
                attack *= 3;
                attack /= 2;
            }
            Ability::GorillaTactics => {
                attack *= 3;
                attack /= 2;
            }
            Ability::Guts => if pokemon.non_volatile_status.is_some() {
                attack *= 3;
                attack /= 2;
            }
            Ability::Hustle => {
                attack *= 3;
                attack /= 2;
            }
            Ability::OrichalcumPulse => if conditions.is_sunny() {
                attack *= 5461;
                attack /= 4096;
            }
            Ability::PurePower | Ability::HugePower => attack *= 2,
            _ => {}
        }

        // thick club
        if let Some(item) = pokemon.item {
            match item {
                Item::ChoiceBand => {
                    attack *= 3;
                    attack /= 2;
                }
                Item::LightBall if pokemon.id == POKEMON_PIKACHU && self.generation >= 5 => attack *= 2,
                _ => {}
            }
        }

        if pokemon.non_volatile_status == Some(NonVolatileStatus::Burn) && ability != Ability::Guts {
            attack /= 2;
        }

        attack

    }

    fn get_defense_stat(&self, pokemon: &Pokemon, conditions: &Conditions) -> u16 {

        let mut defense = pokemon.get_stat(Stat::Defense);

        match self.get_ability(pokemon, conditions) {
            Ability::FurCoat => defense *= 2,
            Ability::GrassPelt => if conditions.is_terrain(Terrain::Grassy) {
                defense *= 3;
                defense /= 2;
            }
            Ability::MarvelScale => if pokemon.non_volatile_status.is_some() {
                defense *= 3;
                defense /= 2;
            }
            _ => {}
        }

        // eviolite

        if let Some(item) = pokemon.item {
            match item {
                Item::MetalPowder if pokemon.id == POKEMON_DITTO => {
                    if self.generation == 2 {
                        defense *= 3;
                        defense /= 2;
                    }
                    // account for transform
                    else {
                        defense *= 2;
                    }
                }
                _ => {}
            }
        }

        defense

    }

    fn get_special_attack_stat(&self, pokemon: &Pokemon, conditions: &Conditions) -> u16 {

        let mut special_attack = pokemon.get_stat(Stat::SpecialAttack);

        // plus/minus

        match self.get_ability(pokemon, conditions) {
            Ability::HadronEngine => if conditions.is_terrain(Terrain::Electric) {
                special_attack *= 5461;
                special_attack /= 4096;
            }
            Ability::SolarPower => if conditions.is_sunny() {
                special_attack *= 3;
                special_attack /= 2;
            }
            _ => {}
        }

        // soul dew
        if let Some(item) = pokemon.item {
            match item {
                Item::ChoiceSpecs => {
                    special_attack *= 3;
                    special_attack /= 2;
                }
                Item::DeepSeaScale if pokemon.id == POKEMON_CLAMPERL => special_attack *= 2,
                Item::LightBall if pokemon.id == POKEMON_PIKACHU && self.generation != 4 => special_attack *= 2,
                _ => {}
            }
        }

        special_attack

    }

    fn get_special_defense_stat(&self, pokemon: &Pokemon, conditions: &Conditions) -> u16 {

        let mut special_defense = pokemon.get_stat(Stat::SpecialDefense);

        match self.get_ability(pokemon, conditions) {
            Ability::FlowerGift => if conditions.is_sunny() {
                special_defense *= 3;
                special_defense /= 2;
            }
            _ => {}
        }

        // eviolite

        if let Some(item) = pokemon.item {
            match item {
                Item::AssaultVest => {
                    special_defense *= 3;
                    special_defense /= 2;
                }
                Item::DeepSeaScale if pokemon.id == POKEMON_CLAMPERL => special_defense *= 2,
                Item::MetalPowder if pokemon.id == POKEMON_DITTO && self.generation == 2 => {
                    special_defense *= 3;
                    special_defense /= 2;
                }
                _ => {}
            }
        }

        if self.generation >= 4 && conditions.is_weather(Weather::Sandstorm) && self.pokemon_has_type(pokemon, Type::Rock) {
            special_defense *= 3;
            special_defense /= 2;
        }

        special_defense

    }

    fn get_speed_stat(&self, pokemon: &Pokemon, conditions: &Conditions) -> u16 {

        let mut speed = pokemon.get_stat(Stat::Speed);

        let ability = self.get_ability(pokemon, conditions);

        match ability {
            Ability::Chlorophyll if conditions.is_sunny() => speed *= 2,
            Ability::QuickFeet if pokemon.non_volatile_status.is_some() => speed *= 2,
            Ability::SandRush if conditions.is_weather(Weather::Sandstorm) => speed *= 2,
            Ability::SlushRush if conditions.is_weather(Weather::Hail) => speed *= 2,
            Ability::SurgeSurfer if conditions.is_terrain(Terrain::Electric) => speed *= 2,
            Ability::SwiftSwim if conditions.is_rain() => speed *= 2,
            _ => {}
        }

        if pokemon.non_volatile_status == Some(NonVolatileStatus::Paralysis) && ability != Ability::QuickFeet {
            if self.generation <= 6 {
                speed /= 4;
            }
            else {
                speed /= 2;
            }
        }

        if let Some(item) = pokemon.item {
            match item {
                Item::ChoiceScarf => {
                    speed *= 3;
                    speed /= 2;
                }
                // account for transform
                Item::QuickPowder if pokemon.id == POKEMON_DITTO => speed *= 2,
                _ => {}
            }
        }

        speed

    }
    
    fn get_priority(&self, action: &BattleAction) -> i8 {
        match action {
            BattleAction::Move(move_id) => self.data_handler.get_move(*move_id).priority,
            BattleAction::Switch(_) => 8,
            BattleAction::Item(_) => 8
        }
    }

    fn get_attacking_stat(&self, used_move: &Move, attacker: &Pokemon, defender: &Pokemon, conditions: &Conditions) -> u16 {

        // if used_move.has_effect(MoveEffect::BodyPress) {
        //     attacker.defense
        // } else
        if used_move.effect == MoveEffect::FoulPlay {
            self.get_attack_stat(defender, &conditions)
        }
        else if used_move.class == MoveClass::Physical {
            self.get_attack_stat(attacker, &conditions)
        }
        else {
            self.get_special_attack_stat(attacker, conditions)
        }
    }

    fn get_defending_stat(&self, used_move: &Move, attacker: &Pokemon, defender: &Pokemon, conditions: &Conditions) -> u16 {

        if used_move.class == MoveClass::Physical || used_move.effect == MoveEffect::Psyshock {
            self.get_defense_stat(defender, conditions)
        }
        else {
            self.get_special_defense_stat(defender, conditions)
        }
    }

    fn get_move_power(&self, used_move: &Move, attacker: &Pokemon, defender: &Pokemon, conditions: &Conditions) -> u32 {

        let mut power = used_move.power.unwrap_or(0) as u32;

        let attacker_ability = self.get_ability(attacker, conditions);

        if used_move.effect == MoveEffect::Return {
            power = (attacker.friendship as u32) * 5 / 2;
        }
        else if used_move.effect == MoveEffect::Frustration {
            power = (255 - attacker.friendship as u32) * 5 / 2;
        }

        let mut modifier = 4096;

        if
            used_move.effect == MoveEffect::Acrobatics && attacker.item.is_none() ||
            used_move.effect == MoveEffect::Brine && defender.hp <= defender.max_hp / 2 ||
            used_move.effect == MoveEffect::Facade && attacker.non_volatile_status.is_some() ||
            used_move.effect == MoveEffect::Venoshock && matches!(defender.non_volatile_status, Some(NonVolatileStatus::Poison | NonVolatileStatus::BadlyPoison))
            // retaliate
            // fusion moves
        {
            modifier *= 2;
        }

        // solarbeam in weather

        // charge

        // helping hand

        // me first

        // sports

        if attacker_ability == Ability::Rivalry {
            if attacker.gender.is_same(defender.gender) {
                modifier *= 5;
                modifier /= 4;
            }
            else if attacker.gender.is_opposite(defender.gender) {
                modifier *= 3;
                modifier /= 4;
            }
        }

        // reckless

        if attacker_ability == Ability::IronFist && used_move.flags.get_punch() {
            modifier *= 4915;
            modifier /= 4096;
        }

        // analytic

        // sand force

        // sheer force

        // strong jaw
        
        if used_move.flags.get_pulse() {
            modifier *= 3;
            modifier /= 2;
        }

        if power <= 60 && attacker_ability == Ability::Technician {
            modifier *= 3;
            modifier /= 2;
        }

        // toxic boost

        // flare boost

        // heatproof

        // dry skin

        // muscle band/wise glasses

        // incense/plate

        if let Some(item) = attacker.item {

            if matches!((used_move.class, item),
                (MoveClass::Physical,   Item::MuscleBand) |
                (MoveClass::Special,    Item::WiseGlasses)
            ) {
                power *= 4505;
                power /= 4096;
            }

            if matches!((used_move.move_type, item), 
                (Type::Dark,        Item::BlackGlasses | Item::DreadPlate) |
                (Type::Dragon,      Item::DragonFang | Item::DracoPlate) |
                (Type::Electric,    Item::Magnet | Item::ZapPlate) |
                (Type::Fairy,       Item::FairyFeather | Item::PixiePlate) |
                (Type::Fighting,    Item::BlackBelt | Item::FistPlate) |
                (Type::Fire,        Item::Charcoal | Item::FlamePlate) |
                (Type::Flying,      Item::SharpBeak | Item::SkyPlate) |
                (Type::Ghost,       Item::SpellTag | Item::SpookyPlate) |
                (Type::Grass,       Item::MiracleSeed | Item::MeadowPlate) |
                (Type::Ground,      Item::SoftSand | Item::EarthPlate) |
                (Type::Ice,         Item::NeverMeltIce | Item::IciclePlate) |
                (Type::Normal,      Item::SilkScarf) | 
                (Type::Poison,      Item::PoisonBarb | Item::ToxicPlate) |
                (Type::Psychic,     Item::TwistedSpoon | Item::MindPlate) |
                (Type::Rock,        Item::HardStone | Item::StonePlate) |
                (Type::Water,       Item::MysticWater | Item::SplashPlate)
            ) {
                power *= 4505;
                power /= 4096;
            }

            if item == Item::LightBall && attacker.id == POKEMON_PIKACHU && self.generation == 4 {
                power *= 2;
            }
        }

        // orbs

        // gems

        power * modifier / 4096

    }

    fn apply_item_boosts(&self, damage: &mut u32, used_move: &Move, attacker: &Pokemon) {

        let Some(item) = attacker.item
        else {
            return;
        };

        if item == Item::LifeOrb {
            *damage *= 8;
            *damage /= 5;
        }
    }

    fn do_contact(&self, attacker: &mut Pokemon, defender: &mut Pokemon, conditions: &Conditions) {

        // baneful bunker, beak blast, burning bulwark, kings shield, obstruct, silk trap, spiky shield

        match self.get_ability(defender, conditions) {
            // Ability::CuteCharm if rand::random_ratio(3, 10) => attacker.volatile_status.add(VolatileStatusEffect::Infatuation),
            Ability::EffectSpore if rand::random_ratio(3, 10) && attacker.non_volatile_status.is_none() =>
                attacker.non_volatile_status = Some([NonVolatileStatus::Poison, NonVolatileStatus::Paralysis, NonVolatileStatus::Sleep][rand::random_range(0..3)]),
            Ability::FlameBody if rand::random_ratio(3, 10) && attacker.non_volatile_status.is_none() =>
                attacker.non_volatile_status = Some(NonVolatileStatus::Burn),
            Ability::Gooey | Ability::TanglingHair => attacker.apply_stat_changes(Stat::Speed, -1),
            Ability::IronBarbs | Ability::RoughSkin => defender.deal_damage(defender.max_hp / 8),
            Ability::Mummy => attacker.volatile_status.add(VolatileStatusEffect::AbilityChange(Ability::Mummy)),
            Ability::PerishBody => todo!(),
            Ability::Pickpocket => todo!(),
            Ability::PoisonPoint if rand::random_ratio(3, 10) && attacker.non_volatile_status.is_none() =>
                attacker.non_volatile_status = Some(NonVolatileStatus::Poison),
            Ability::PoisonTouch if rand::random_ratio(3, 10) && defender.non_volatile_status.is_none() =>
                defender.non_volatile_status = Some(NonVolatileStatus::Poison),
            Ability::Static if rand::random_ratio(3, 10) && defender.non_volatile_status.is_none() =>
                defender.non_volatile_status = Some(NonVolatileStatus::Paralysis),
            Ability::WanderingSpirit => todo!(),
            _ => {}
        }



    }

    //    Damage=((2×Level5+2)×Power×AD50+2)×Targets×PB×Weather×GlaiveRush×Critical×random×STAB×Type×Burn×other×ZMove×TeraShield
    pub fn calc_damage_inner(&self, used_move: &Move, attacker: &Pokemon, defender: &Pokemon, conditions: &Conditions, hit_properties: HitProperties) -> u16 {

        let attack_stat = self.get_attacking_stat(used_move, attacker, defender, conditions) as u32;
        let defense_stat = self.get_defending_stat(used_move, attacker, defender, conditions) as u32;

        let attacker_data = self.data_handler.get_pokemon_data(attacker.id);
        let defender_data = self.data_handler.get_pokemon_data(defender.id);

        let power = self.get_move_power(used_move, attacker, defender, conditions);

        let mut damage = (2 * (attacker.level as u32) / 5 + 2) * power * attack_stat / defense_stat / 50 + 2;

        // targets
        if false {
            damage *= 3;
            damage /= 4;
        }

        // parental bond
        if false {
            damage /= 4;
        }

        // weather
        if false {
        }

        // glaive rush
        if false {
            damage *= 2;
        }

        if hit_properties.crit {
            damage *= 2;
        }

        damage *= hit_properties.roll_percent as u32;
        damage /= 100;

        // stab
        if attacker_data.is_type(used_move.move_type) {
            damage *= 3;
            damage /= 2;
        }

        let mut type_effectiveness = self.data_handler.type_chart.get(used_move.move_type, defender_data.type1) as u32;

        if let Some(type2) = defender_data.type2 {
            type_effectiveness *= self.data_handler.type_chart.get(used_move.move_type, type2) as u32;
            type_effectiveness /= 100;
        }

        if type_effectiveness > 100 {
            log!("It's super effective!");
        }
        else if type_effectiveness == 0 {
            log!("It doesn't affect...");
        }
        else if type_effectiveness < 100 {
            log!("It's not very effective...");
        }

        damage *= type_effectiveness;
        damage /= 100;

        // burn
        if used_move.class == MoveClass::Physical && attacker.non_volatile_status == Some(NonVolatileStatus::Burn) {
            damage /= 2;
        }

        // tons of random stuff
        self.apply_item_boosts(&mut damage, used_move, attacker);

        let mut damage_u16 = damage as u16;

        if used_move.effect == MoveEffect::FalseSwipe && damage_u16 >= defender.hp {
            damage_u16 = defender.hp;
        }
        
        damage_u16

    }

    fn calc_damage(&self, used_move: &Move, attacker: &Pokemon, defender: &Pokemon, conditions: &Conditions) -> u16 {

        let crit = rand::random_ratio(1, 16);

        if crit {
            log!("A critical hit!");
        }

        let roll_percent = rand::random_range(85..=100);
        
        let hit_properties = HitProperties {
            crit,
            roll_percent
        };

        self.calc_damage_inner(used_move, attacker, defender, conditions, hit_properties)
        
    }

    fn do_move_hit(&self, used_side: &mut Side, other_side: &mut Side, mut damage: u16, used_move: &Move, conditions: &Conditions) {

        let attacker = used_side.get_active_mut();
        let defender = other_side.get_active_mut();

        let defender_ability = self.get_ability(&defender, conditions);

        if damage >= defender.hp {

            if defender.hp == defender.max_hp {
                if defender_ability == Ability::Sturdy {
                    damage = defender.hp - 1;
                }
                else if defender.item == Some(Item::FocusSash) {
                    log!("{} held on using their Focus Sash!", defender.name);
                    damage = defender.hp - 1;
                    defender.item = None;
                }
            }

            if defender.item == Some(Item::FocusBand) && rand::random_ratio(1, 10) {
                log!("{} held on using their Focus Band!", defender.name);
                damage = defender.hp - 1;
            }
        }

        defender.deal_damage(damage);

        if defender.hp == 0 {
            log!("{} fainted!", defender.name);
        }

        if used_move.move_type == Type::Dark && defender_ability == Ability::Justified {
            defender.apply_stat_changes(Stat::Attack, 1);
        }

        if used_move.flags.get_contact() && attacker.item == Some(Item::ProtectivePads) && self.get_ability(attacker, conditions) != Ability::LongReach {
            self.do_contact(attacker, defender, conditions);
        }
    }

    fn use_move(&self, used_move: &Move, using_side: &mut Side, other_side: &mut Side, conditions: &mut Conditions) {
        
        log!("{} used {}!", using_side.get_active().name, used_move.name);

        let mut damage = 0;

        if used_move.class != MoveClass::Status {
            damage = self.calc_damage(used_move, &using_side.get_active_mut(), &other_side.get_active_mut(), conditions);
            self.do_move_hit(using_side, other_side, damage, used_move, conditions);
        }

        let mut do_effect = true;

        if let Some(chance) = used_move.effect_chance {
            do_effect = rand::random_ratio(chance as u32, 100);
        }

        if do_effect {
            self.apply_effect_after_use(used_move.effect, using_side, other_side, conditions, damage);
        }
    }

    fn set_weather(&self, conditions: &mut Conditions, weather: Weather, held_item: Option<Item>, from_ability: bool) {
        
        if let Some((current_weather, _)) = conditions.weather {
            if weather == current_weather || current_weather.is_strong() && !weather.is_strong() {
                return;
            }
        }

        let duration = if self.generation <= 5 && from_ability {
            Weather::PERMANENT
        }
        else if matches!(weather, Weather::ExtremeSun | Weather::HeavyRain | Weather::StrongWind) {
            Weather::PERMANENT
        }
        else if held_item.is_some_and(|item| matches!((weather, item), 
            (Weather::Rain, Item::DampRock) |
            (Weather::Sun, Item::HeatRock) |
            (Weather::Hail, Item::IcyRock) |
            (Weather::Sandstorm, Item::SmoothRock)
        )) {
            8
        }
        else {
            5
        };

        conditions.weather = Some((weather, duration));

    }

    fn set_terrain(&self, conditions: &mut Conditions, terrain: Terrain, held_item: Option<Item>) {

        let duration = if held_item == Some(Item::TerrainExtender) {
            8
        }
        else {
            5
        };

        conditions.terrain = Some((terrain, duration));

    }

    fn activate_ability(&self, side: &mut Side, other_side: &mut Side, conditions: &mut Conditions) {

        let mon = side.get_active_mut();
        let other_mon = other_side.get_active_mut();

        match self.get_ability(mon, &conditions) {
            Ability::AirLock | Ability::CloudNine => todo!(),
            Ability::Anticipation => todo!(),
            // Ability::AsOne => todo!(),
            Ability::CuriousMedicine => todo!(),
            Ability::DauntlessShield => mon.apply_stat_changes(Stat::Defense, 1), // gen 9, only once per battle
            Ability::DeltaStream => self.set_weather(conditions, Weather::StrongWind, mon.item, true),
            Ability::DesolateLand => conditions.weather = Some((Weather::ExtremeSun, Weather::PERMANENT)),
            Ability::Download => {
                // need to account for gen 4 weirdness
                let other_mon = other_side.get_active();
                if self.get_defense_stat(other_mon, &conditions) < self.get_special_defense_stat(other_mon, &conditions) {
                    mon.apply_stat_changes(Stat::Attack, 1);
                }
                else {
                    mon.apply_stat_changes(Stat::SpecialAttack, 1);
                }
            }
            Ability::Drizzle => self.set_weather(conditions, Weather::Rain, mon.item, true),
            Ability::Drought | Ability::OrichalcumPulse => self.set_weather(conditions, Weather::Sun, mon.item, true),
            Ability::ElectricSurge | Ability::HadronEngine => self.set_terrain(conditions, Terrain::Electric, mon.item),
            Ability::Forewarn => todo!(),
            Ability::Frisk => todo!(),
            Ability::GrassySurge => self.set_terrain(conditions, Terrain::Grassy, mon.item),
            Ability::Hospitality => todo!(),
            Ability::Imposter => todo!(),
            Ability::Intimidate => other_mon.apply_stat_changes(Stat::Attack, -1),
            Ability::IntrepidSword => mon.apply_stat_changes(Stat::Attack, 1), // gen 9, only once per battle
            Ability::MistySurge => self.set_terrain(conditions, Terrain::Misty, mon.item),
            Ability::MoldBreaker => todo!(),
            Ability::NeutralizingGas => todo!(),
            Ability::Pressure => todo!(),
            Ability::PrimordialSea => self.set_weather(conditions, Weather::HeavyRain, mon.item, true),
            Ability::Protosynthesis => todo!(),
            Ability::PsychicSurge => self.set_terrain(conditions, Terrain::Psychic, mon.item),
            Ability::QuarkDrive => todo!(),
            Ability::SandStream => self.set_weather(conditions, Weather::Sandstorm, mon.item, true),
            Ability::Schooling => todo!(),
            Ability::ScreenCleaner => todo!(),
            Ability::ShieldsDown => todo!(),
            Ability::SnowWarning => self.set_weather(conditions, Weather::Hail, mon.item, true),
            Ability::SupersweetSyrup => other_mon.apply_stat_changes(Stat::Evasion, -1),
            Ability::SupremeOverlord => todo!(),
            Ability::Trace => {
                mon.volatile_status.add(VolatileStatusEffect::AbilityChange(self.get_ability(other_side.get_active(), &conditions)));
                self.activate_ability(side, other_side, conditions);
            }
            Ability::Unnerve => todo!(),
            _ => {}
        }
    }

    fn apply_effect_after_use(&self, effect: MoveEffect, user_side: &mut Side, target_side: &mut Side, conditions: &mut Conditions, move_damage: u16) {

        let using_mon = user_side.get_active_mut();
        let target_mon = target_side.get_active_mut();

        use MoveEffect as ME;
        match effect {
            ME::Sleep => target_side.try_apply_status(NonVolatileStatus::Sleep),
            ME::PoisonChance => target_side.try_apply_status(NonVolatileStatus::Poison),
            ME::DrainHalf => using_mon.heal(move_damage / 2),
            ME::Burn | ME::BurnChance => target_side.try_apply_status(NonVolatileStatus::Burn),
            ME::FreezeChance => target_side.try_apply_status(NonVolatileStatus::Freeze),
            ME::Paralyze | ME::ParalyzeChance => target_side.try_apply_status(NonVolatileStatus::Paralysis),
            ME::FaintUser => using_mon.hp = 0,
            ME::DreamEater => todo!(),
            ME::UseTargetsLastMove => todo!(),
            ME::RaiseUserAttack1 => using_mon.apply_stat_changes(Stat::Attack, 1),
            ME::RaiseUserDefense1 => using_mon.apply_stat_changes(Stat::Defense, 1),
            ME::RaiseUserSpecialAttack1 => using_mon.apply_stat_changes(Stat::SpecialAttack, 1),
            //ME::RaiseUserSpecialDefense1 => using_mon.apply_stat_changes(Stat::SpecialDefense, 1),
            ME::RaiseUserSpeed1 => using_mon.apply_stat_changes(Stat::Speed, 1),
            ME::NeverMiss => todo!(),
            ME::LowerTargetAttack1 => target_mon.apply_stat_changes(Stat::Attack, -1),
            ME::LowerTargetDefense1 => target_mon.apply_stat_changes(Stat::Defense, -1),
            // ME::LowerTargetSpecialAttack1 => using_mon.apply_stat_changes(Stat::SpecialAttack, 1),
            // ME::LowerTargetSpecialDefense1 => using_mon.apply_stat_changes(Stat::SpecialDefense, 1),
            ME::LowerTargetSpeed1 => target_mon.apply_stat_changes(Stat::Speed, -1),
            ME::LowerTargetAccuracy1 => target_mon.apply_stat_changes(Stat::Accuracy, -1),
            ME::LowerTargetEvasion1 => target_mon.apply_stat_changes(Stat::Evasion, -1),
            ME::ResetTargetStats => target_mon.reset_stat_changes(),
            ME::Bide => todo!(),
            ME::ForceSwitch => todo!(),
            ME::Hit2To5Times => todo!(),
            ME::Conversion => todo!(),
            ME::FlinchChance => todo!(),
            ME::HealUserHalf => todo!(),
            ME::BadlyPoison | ME::BadlyPoisonChance => target_side.try_apply_status(NonVolatileStatus::BadlyPoison),
            ME::ScatterMoney => todo!(),
            ME::LightScreen => target_side.effects.set_light_screen(if using_mon.item == Some(Item::LightClay) {8} else {5}),
            ME::TriAttack => todo!(),
            ME::Rest => todo!(),
            ME::RazorWind => todo!(),
            ME::SuperFang => todo!(),
            ME::DragonRage => todo!(),
            ME::Trapping => todo!(),
            ME::IncreasedCrit => todo!(),
            ME::HitTwice => todo!(),
            ME::RecoilOnMiss => todo!(),
            ME::ProtectStats => todo!(),
            ME::FocusEnergy => todo!(),
            ME::RecoilQuarter => todo!(),
            ME::Confuse | ME::ConfuseAllTargets | ME::ConfuseChance => target_mon.volatile_status.add(VolatileStatusEffect::Confusion(rand::random_range(2..=5))),
            ME::RaiseUserAttack2 => using_mon.apply_stat_changes(Stat::Attack, 2),
            ME::RaiseUserDefense2 => using_mon.apply_stat_changes(Stat::Defense, 2),
            ME::RaiseUserSpeed2 => using_mon.apply_stat_changes(Stat::Speed, 2),
            ME::RaiseUserSpecialAttack2 => using_mon.apply_stat_changes(Stat::SpecialAttack, 2),
            ME::RaiseUserSpecialDefense2 => using_mon.apply_stat_changes(Stat::SpecialDefense, 2),
            ME::Transform => todo!(),
            ME::LowerTargetAttack2 => target_mon.apply_stat_changes(Stat::Attack, 2),
            ME::LowerTargetDefense2 => target_mon.apply_stat_changes(Stat::Defense, 2),
            ME::LowerTargetSpeed2 => target_mon.apply_stat_changes(Stat::Speed, 2),
            ME::LowerTargetSpecialAttack2 => target_mon.apply_stat_changes(Stat::SpecialAttack, 2),
            ME::LowerTargetSpecialDefense2 => target_mon.apply_stat_changes(Stat::SpecialDefense, 2),
            ME::Reflect => target_side.effects.set_reflect(if using_mon.item == Some(Item::LightClay) {8} else {5}),
            ME::Poison => target_side.try_apply_status(NonVolatileStatus::Poison),
            ME::LowerTargetAttack1Chance => target_mon.apply_stat_changes(Stat::Attack, -1),
            ME::LowerTargetDefense1Chance => target_mon.apply_stat_changes(Stat::Defense, -1),
            ME::LowerTargetSpeed1Chance => target_mon.apply_stat_changes(Stat::Speed, -1),
            ME::LowerTargetSpecialAttack1Chance => target_mon.apply_stat_changes(Stat::SpecialAttack, -1),
            ME::LowerTargetSpecialDefense1Chance => target_mon.apply_stat_changes(Stat::SpecialDefense, -1),
            ME::LowerTargetAccuracy1Chance => target_mon.apply_stat_changes(Stat::Accuracy, -1),
            ME::MysticalFire => todo!(),
            ME::ChargeAndFlinchChance => todo!(),
            ME::HitTwiceAndPoisonChance => todo!(),
            ME::Substitute => todo!(),
            ME::Recharge => todo!(),
            ME::RaiseAttack1IfHit => todo!(),
            ME::UseTargetsLastMove2 => todo!(),
            ME::Metronome => todo!(),
            ME::Seed => todo!(),
            ME::Splash => todo!(),
            ME::Disable => todo!(),
            ME::DamageByLevel => todo!(),
            ME::RangeDamageByLevel => todo!(),
            ME::Counter => todo!(),
            ME::Encore => todo!(),
            ME::PainSplit => {
                let total = using_mon.hp + target_mon.hp;
                using_mon.hp = using_mon.max_hp.min(total / 2);
                target_mon.hp = target_mon.max_hp.min(total / 2);
            },
            ME::FlinchChanceWorksIfSleeping => todo!(),
            ME::Transform2 => todo!(),
            ME::NextMoveHits => todo!(),
            ME::Sketch => todo!(),
            ME::SleepTalk => todo!(),
            ME::DestinyBond => todo!(),
            ME::MorePowerWhenLessHP => todo!(),
            ME::Spite => todo!(),
            ME::CurePartyStatus => todo!(),
            ME::NoOtherEffect2 => todo!(),
            ME::Hit3TimesIncreasing => todo!(),
            ME::StealItem => todo!(),
            ME::PreventEscape => todo!(),
            ME::Nightmare => todo!(),
            ME::Minimize => todo!(),
            ME::Curse => todo!(),
            ME::Protect => todo!(),
            ME::Spikes => target_side.effects.add_spikes(),
            ME::Identify => todo!(),
            ME::PerishSong => todo!(),
            ME::Sandstorm => self.set_weather(conditions, Weather::Sandstorm, using_mon.item, false),
            ME::Endure => todo!(),
            ME::Rollout => todo!(),
            ME::Swagger => todo!(),
            ME::IceBall => todo!(),
            ME::Attract => todo!(),
            ME::Return => todo!(),
            ME::Present => todo!(),
            ME::Frustration => todo!(),
            ME::Safeguard => todo!(),
            ME::Magnitude => todo!(),
            ME::BatonPass => todo!(),
            ME::Pursuit => todo!(),
            ME::RapidSpin => todo!(),
            ME::SonicBoom => todo!(),
            ME::Moonlight => todo!(),
            ME::HiddenPower => todo!(),
            ME::RainDance => self.set_weather(conditions, Weather::Rain, using_mon.item, false),
            ME::SunnyDay => self.set_weather(conditions, Weather::Sun, using_mon.item, false),
            ME::RaiseUserDefense1Chance => todo!(),
            ME::RaiseUserAttack1Chance => todo!(),
            ME::RaiseAllUserStats1Chance => todo!(),
            ME::BellyDrum => todo!(),
            ME::PsychUp => todo!(),
            ME::MirrorCoat => todo!(),
            ME::SkullBash => todo!(),
            ME::Twister => todo!(),
            ME::Earthquake => todo!(),
            ME::Hits2TurnsLater => todo!(),
            ME::Gust => todo!(),
            ME::Stomp => todo!(),
            ME::Solarbeam => todo!(),
            ME::Thunder => todo!(),
            ME::Teleport => todo!(),
            ME::BeatUp => todo!(),
            ME::Fly => todo!(),
            ME::DefenseCurl => todo!(),
            ME::FakeOut => todo!(),
            ME::Uproar => todo!(),
            ME::Stockpile => todo!(),
            ME::SpitUp => todo!(),
            ME::Swallow => todo!(),
            ME::Hail => self.set_weather(conditions, Weather::Hail, using_mon.item, false),
            ME::Torment => todo!(),
            ME::Flatter => todo!(),
            ME::Memento => todo!(),
            ME::Facade => todo!(),
            ME::FocusPunch => todo!(),
            ME::SmellingSalts => todo!(),
            ME::FollowMe => todo!(),
            ME::NaturePower => todo!(),
            ME::Charge => todo!(),
            ME::Taunt => todo!(),
            ME::HelpingHand => todo!(),
            ME::Trick => todo!(),
            ME::RolePlay => todo!(),
            ME::Wish => todo!(),
            ME::RandomlySwitchOutTarget => todo!(),
            ME::Ingrain => todo!(),
            ME::LowerUserAttackDefense1 => todo!(),
            ME::MagicCoat => todo!(),
            ME::Recycle => todo!(),
            ME::DoubleDamageIfHitBeforeAttacking => todo!(),
            ME::DestroyScreens => todo!(),
            ME::Yawn => todo!(),
            ME::KnockOff => todo!(),
            ME::PowerBasedOnUserHP => todo!(),
            ME::SkillSwap => todo!(),
            ME::Imprison => todo!(),
            ME::HealUserStatus => todo!(),
            ME::Grudge => todo!(),
            ME::Snatch => todo!(),
            ME::PowerBasedOnWeight => todo!(),
            ME::SecretPower => todo!(),
            ME::RecoilThird => todo!(),
            ME::IncreasedCritAndBurnChance => todo!(),
            ME::MudSport => todo!(),
            ME::WeatherBall => todo!(),
            ME::LowerUserSpecialAttack => todo!(),
            ME::LowerTargetAttackDefense1 => todo!(),
            ME::RaiseUserDefenseSpecialDefense1 => todo!(),
            ME::HitBounceFly => todo!(),
            ME::RaiseUserAttackDefense1 => todo!(),
            ME::IncreasedCritAndPoisonChance => todo!(),
            ME::WaterSport => todo!(),
            ME::RaiseUserSpecialAttackSpecialDefense1 => todo!(),
            ME::DragonDance => {
                using_mon.apply_stat_changes(Stat::Attack, 1);
                using_mon.apply_stat_changes(Stat::Speed, 1);
            }
            ME::Camouflage => todo!(),
            ME::Roost => todo!(),
            ME::Gravity => todo!(),
            ME::MiracleEye => todo!(),
            ME::WakeUpSlap => todo!(),
            ME::LowerUserSpeed1 => using_mon.apply_stat_changes(Stat::Speed, -1),
            ME::GyroBall => todo!(),
            ME::HealingWish => todo!(),
            ME::NaturalGift => todo!(),
            ME::Feint => todo!(),
            ME::Pluck => todo!(),
            ME::Tailwind => user_side.effects.set_tailwind(5),
            ME::Acupressure => todo!(),
            ME::MetalBurst => todo!(),
            ME::SwitchAfterAttacking => todo!(),
            ME::LowerUserDefenseSpecialDefense1 => todo!(),
            ME::DoublePowerIfTargetAlreadyMoved => todo!(),
            ME::DoublePowerIfTargetAlreadyTookDamage => todo!(),
            ME::Embargo => todo!(),
            ME::Fling => todo!(),
            ME::PsychoShift => todo!(),
            ME::TrumpCard => todo!(),
            ME::HealBlock => todo!(),
            ME::PowerTrick => todo!(),
            ME::GastroAcid => target_mon.volatile_status.add(VolatileStatusEffect::AbilitySuppression),
            ME::LuckyChant => todo!(),
            ME::MeFirst => todo!(),
            ME::Copycat => todo!(),
            ME::PowerSwap => todo!(),
            ME::GuardSwap => todo!(),
            ME::Punishment => todo!(),
            ME::LastResort => todo!(),
            ME::WorrySeed => todo!(),
            ME::SuckerPunch => todo!(),
            ME::ToxicSpikes => target_side.effects.add_toxic_spikes(),
            ME::HeartSwap => todo!(),
            ME::AquaRing => todo!(),
            ME::MagnetRise => todo!(),
            ME::FlareBlitz => todo!(),
            ME::Struggle => todo!(),
            ME::Dive => todo!(),
            ME::Dig => todo!(),
            ME::Defog => todo!(),
            ME::TrickRoom => todo!(),
            ME::Blizzard => todo!(),
            ME::Whirlpool => todo!(),
            ME::VoltTackle => todo!(),
            ME::Bounce => todo!(),
            ME::Captivate => todo!(),
            ME::StealthRock => target_side.effects.set_stealth_rock(true),
            ME::Chatter => todo!(),
            ME::RecoilHalf => todo!(),
            ME::LunarDance => todo!(),
            ME::LowerTargetSpecialDefense2Chance => todo!(),
            ME::Disappear1TurnIgnoreProtect => todo!(),
            ME::FireFang => todo!(),
            ME::IceFang => todo!(),
            ME::ThunderFang => todo!(),
            ME::RaiseUserSpecialAttack1Chance => todo!(),
            ME::HoneClaws => todo!(),
            ME::WideGuard => todo!(),
            ME::GuardSplit => todo!(),
            ME::PowerSplit => todo!(),
            ME::WonderRoom => todo!(),
            ME::Psyshock => todo!(),
            ME::Venoshock => todo!(),
            ME::Autotomize => todo!(),
            ME::Telekinesis => todo!(),
            ME::MagicRoom => todo!(),
            ME::SmackDown => todo!(),
            ME::AlwaysCrits => todo!(),
            ME::SplashDamage => todo!(),
            ME::QuiverDance => {
                using_mon.apply_stat_changes(Stat::SpecialAttack, 1);
                using_mon.apply_stat_changes(Stat::SpecialDefense, 1);
                using_mon.apply_stat_changes(Stat::Speed, 1);
            }
            ME::HeavySlam => todo!(),
            ME::HitIfTypesShared => todo!(),
            ME::PowerBasedOnSpeedDifference => todo!(),
            ME::Soak => todo!(),
            ME::AcidSpray => todo!(),
            ME::SimpleBeam => todo!(),
            ME::Entrainment => todo!(),
            ME::AfterYou => todo!(),
            ME::Round => todo!(),
            ME::EchoedVoice => todo!(),
            ME::DarkestLariat => todo!(),
            ME::ClearSmog => todo!(),
            ME::PowerTrip => todo!(),
            ME::QuickGuard => todo!(),
            ME::AllySwitch => todo!(),
            ME::ShellSmash => {
                using_mon.apply_stat_changes(Stat::Attack, 2);
                using_mon.apply_stat_changes(Stat::SpecialAttack, 2);
                using_mon.apply_stat_changes(Stat::Speed, 2);
                using_mon.apply_stat_changes(Stat::Defense, -2);
                using_mon.apply_stat_changes(Stat::SpecialDefense, -2);
            },
            ME::HealPulse => todo!(),
            ME::SkyDrop => todo!(),
            ME::ShiftGear => todo!(),
            ME::Roar => todo!(),
            ME::Incinerate => todo!(),
            ME::Quash => todo!(),
            ME::Growth => todo!(),
            ME::ReflectType => todo!(),
            ME::Retaliate => todo!(),
            ME::FinalGambit => todo!(),
            ME::TailGlow => using_mon.apply_stat_changes(Stat::SpecialAttack, 3),
            ME::Coil => {
                using_mon.apply_stat_changes(Stat::Attack, 1);
                using_mon.apply_stat_changes(Stat::Defense, 1);
                using_mon.apply_stat_changes(Stat::Accuracy, 1);
            },
            ME::Thief => todo!(),
            ME::WaterPledge => todo!(),
            ME::FirePledge => todo!(),
            ME::GrassPledge => todo!(),
            ME::WorkUp => {
                using_mon.apply_stat_changes(Stat::Attack, 1);
                using_mon.apply_stat_changes(Stat::SpecialAttack, 1);
            },
            ME::CottonGuard => using_mon.apply_stat_changes(Stat::Defense, 3),
            ME::RelicSong => todo!(),
            ME::RockTomb => target_mon.apply_stat_changes(Stat::Speed, -1),
            ME::FreezeShock => todo!(),
            ME::IceBurn => todo!(),
            ME::Hurricane => todo!(),
            ME::VCreate => {
                using_mon.apply_stat_changes(Stat::Defense, -1);
                using_mon.apply_stat_changes(Stat::SpecialDefense, -1);
                using_mon.apply_stat_changes(Stat::Speed, -1);
            },
            ME::FlyingPress => todo!(),
            ME::Belch => todo!(),
            ME::Rototiller => todo!(),
            ME::StickyWeb => target_side.effects.set_sticky_web(true),
            ME::FellStinger => todo!(),
            _ => {}
        }
    }

    fn do_switch(&self, idx: usize, side: &mut Side, other_side: &mut Side, conditions: &mut Conditions) {

        let mon = side.get_active_mut();

        match self.get_ability(mon, &conditions) {
            Ability::NaturalCure => mon.non_volatile_status = None,
            Ability::Regenerator => mon.heal(mon.max_hp / 3),
            _ => {}
        }

        mon.volatile_status.clear();

        side.active_pokemon = idx;

        self.activate_ability(side, other_side, conditions);

    }

    fn perform_action(&self, action: BattleAction, used_by_side1: bool, state: &mut BattleState) {

        let (using_side, other_side) = if used_by_side1 {
            (&mut state.side1, &mut state.side2)
        }
        else {
            (&mut state.side2, &mut state.side1)
        };

        if using_side.get_active().hp == 0 {
            return;
        }

        match action {
            BattleAction::Move(move_id) => {
                let used_move = self.data_handler.get_move(move_id);
                self.use_move(used_move, using_side, other_side, &mut state.conditions);
            }
            BattleAction::Switch(mon_idx) =>  {
                self.do_switch(mon_idx as usize, using_side, other_side, &mut state.conditions);
            }
            BattleAction::Item(_) => todo!()
        }
    }

    fn on_turn_end(&self, state: &mut BattleState) {
        
        state.side1.get_active_mut().volatile_status.decriment_counters();
        state.side2.get_active_mut().volatile_status.decriment_counters();

        state.conditions.decriment_counters();

    }

    pub fn simulate_turn(&self, side1_action: BattleAction, side2_action: BattleAction, state: &mut BattleState) {
        
        log!("");

        let side1_pokemon = &state.side1.team[state.side1.active_pokemon];
        let side2_pokemon = &state.side2.team[state.side2.active_pokemon];

        let side1_priority = self.get_priority(&side1_action);
        let side2_priority = self.get_priority(&side2_action);

        let mut side1_first = rand::random::<bool>();

        let side1_speed = self.get_speed_stat(side1_pokemon, &state.conditions);
        let side2_speed = self.get_speed_stat(side2_pokemon, &state.conditions);

        if side1_priority != side2_priority {
            side1_first = side1_priority > side2_priority;
        }
        else if side1_speed != side2_speed {
            side1_first = side1_speed != side2_speed
        }

        if side1_first {
            self.perform_action(side1_action, true, state);
            self.perform_action(side2_action, false, state);
        }
        else {
            self.perform_action(side2_action, false, state);
            self.perform_action(side1_action, true, state);
        }

        self.on_turn_end(state);

        // println!("{:#?}", state.side1);
        // println!("{:#?}", state.side2);
        
    }
}