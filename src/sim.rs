use crate::db_enums::*;
use crate::database::*;
use crate::pokemon::*;
use crate::battle::*;

pub struct Simulator<'a> {
    data_handler: &'a DataHandler,
    pub generation: u8
}

impl<'a> Simulator<'a> {

    pub fn new(data_handler: &'a DataHandler, generation: u8) -> Self {
        Self {
            data_handler,
            generation
        }
    }

    fn get_active_ability(&self, pokemon: &Pokemon, state: &BattleState) -> Ability {
        pokemon.ability
    }

    // protosynthesis/quark drive??????

    fn get_attack_stat(&self, pokemon: &Pokemon, state: &BattleState) -> u16 {

        let mut attack = pokemon.attack;

        let ability = self.get_active_ability(pokemon, state);

        match ability {
            Ability::PurePower | Ability::HugePower => attack *= 2,
            Ability::Hustle => {
                attack *= 3;
                attack /= 2;
            }
            Ability::FlowerGift => if state.is_sunny() {
                attack *= 3;
                attack /= 2;
            }
            Ability::GorillaTactics => {
                attack *= 3;
                attack /= 2;
            }
            Ability::Guts => if pokemon.status.is_some() {
                attack *= 3;
                attack /= 2;
            }
            Ability::OrichalcumPulse => if state.is_sunny() {
                attack *= 5461;
                attack /= 4096;
            }
            _ => {}
        }

        // thick club
        if let Some(item) = pokemon.held_item {
            match item {
                Item::ChoiceBand => {
                    attack *= 3;
                    attack /= 2;
                }
                Item::LightBall if pokemon.id == POKEMON_PIKACHU && self.generation >= 5 => attack *= 2,
                _ => {}
            }
        }

        if pokemon.status == Some(NonVolatileStatus::Burn) && ability != Ability::Guts {
            attack /= 2;
        }

        attack

    }

    fn get_defense_stat(&self, pokemon: &Pokemon, state: &BattleState) -> u16 {

        let mut defense = pokemon.defense;

        match self.get_active_ability(pokemon, state) {
            Ability::FurCoat => defense *= 2,
            Ability::GrassPelt => if state.is_terrain(Terrain::Grassy) {
                defense *= 3;
                defense /= 2;
            }
            Ability::MarvelScale => if pokemon.status.is_some() {
                defense *= 3;
                defense /= 2;
            }
            _ => {}
        }

        // eviolite

        if let Some(item) = pokemon.held_item {
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

    fn get_special_attack_stat(&self, pokemon: &Pokemon, state: &BattleState) -> u16 {

        let mut special_attack = pokemon.special_attack;

        // plus/minus

        match self.get_active_ability(pokemon, state) {
            Ability::HadronEngine => if state.is_terrain(Terrain::Electric) {
                special_attack *= 5461;
                special_attack /= 4096;
            }
            Ability::SolarPower => if state.is_sunny() {
                special_attack *= 3;
                special_attack /= 2;
            }
            _ => {}
        }

        // soul dew
        if let Some(item) = pokemon.held_item {
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

    fn get_special_defense_stat(&self, pokemon: &Pokemon, state: &BattleState) -> u16 {

        let mut special_defense = pokemon.special_defense;

        match self.get_active_ability(pokemon, state) {
            Ability::FlowerGift => if state.is_sunny() {
                special_defense *= 3;
                special_defense /= 2;
            }
            _ => {}
        }

        // eviolite

        if let Some(item) = pokemon.held_item {
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

        special_defense

    }

    fn get_speed_stat(&self, pokemon: &Pokemon, state: &BattleState) -> u16 {

        let mut speed = pokemon.speed;

        let ability = self.get_active_ability(pokemon, state);

        match ability {
            Ability::SwiftSwim if state.is_rain() => speed *= 2,
            Ability::Chlorophyll if state.is_sunny() => speed *= 2,
            Ability::SurgeSurfer if state.is_terrain(Terrain::Electric) => speed *= 2,
            Ability::SwiftSwim if state.is_weather(Weather::Hail) => speed *= 2,
            Ability::SandRush if state.is_weather(Weather::Sandstorm) => speed *= 2,
            Ability::QuickFeet if pokemon.status.is_some() => speed *= 2,
            _ => {}
        }

        if pokemon.status == Some(NonVolatileStatus::Paralysis) && ability != Ability::QuickFeet {
            if self.generation <= 6 {
                speed /= 4;
            }
            else {
                speed /= 2;
            }
        }

        if let Some(item) = pokemon.held_item {
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

    fn get_attacking_stat(&self, used_move: &Move, attacker: &Pokemon, defender: &Pokemon, state: &BattleState) -> u16 {

        // if used_move.has_effect(MoveEffect::BodyPress) {
        //     attacker.defense
        // } else
        if used_move.has_effect(MoveEffect::FoulPlay) {
            self.get_attack_stat(defender, &state)
        }
        else if used_move.class == MoveClass::Physical {
            self.get_attack_stat(attacker, &state)
        }
        else {
            self.get_special_attack_stat(attacker, state)
        }
    }

    fn get_defending_stat(&self, used_move: &Move, attacker: &Pokemon, defender: &Pokemon, state: &BattleState) -> u16 {

        if used_move.class == MoveClass::Physical || used_move.has_effect(MoveEffect::Psyshock) {
            self.get_defense_stat(defender, state)
        }
        else {
            self.get_special_defense_stat(defender, state)
        }
    }

    fn get_move_power(&self, used_move: &Move, attacker: &Pokemon, defender: &Pokemon, state: &BattleState) -> u32 {

        let mut power = used_move.power.unwrap_or(0) as u32;

        let attacker_ability = self.get_active_ability(attacker, state);

        if used_move.has_effect(MoveEffect::Return) {
            power = (attacker.friendship as u32) * 5 / 2;
        }
        else if used_move.has_effect(MoveEffect::Frustration) {
            power = (255 - attacker.friendship as u32) * 5 / 2;
        }

        let mut modifier = 4096;

        if
            used_move.has_effect(MoveEffect::Facade) && attacker.status.is_some() ||
            used_move.has_effect(MoveEffect::Brine) && defender.hp <= defender.max_hp / 2 ||
            used_move.has_effect(MoveEffect::Acrobatics) && attacker.held_item.is_none() ||
            used_move.has_effect(MoveEffect::Venoshock) && matches!(defender.status, Some(NonVolatileStatus::Poison | NonVolatileStatus::BadlyPoison))
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

        if let Some(item) = attacker.held_item {

            if matches!((used_move.class, item),
                (MoveClass::Physical,   Item::MuscleBand) |
                (MoveClass::Special,    Item::WiseGlasses)
            ) {
                power *= 4505;
                power /= 4096;
            }

            if matches!((used_move.move_type, item), 
                (Type::Ground,      Item::SoftSand | Item::EarthPlate) |
                (Type::Rock,        Item::HardStone | Item::StonePlate) |
                (Type::Grass,       Item::MiracleSeed | Item::MeadowPlate) |
                (Type::Dark,        Item::BlackGlasses | Item::DreadPlate) |
                (Type::Fighting,    Item::BlackBelt | Item::FistPlate) |
                (Type::Electric,    Item::Magnet | Item::ZapPlate) |
                (Type::Water,       Item::MysticWater | Item::SplashPlate) |
                (Type::Flying,      Item::SharpBeak | Item::SkyPlate) |
                (Type::Poison,      Item::PoisonBarb | Item::ToxicPlate) |
                (Type::Ice,         Item::NeverMeltIce | Item::IciclePlate) |
                (Type::Ghost,       Item::SpellTag | Item::SpookyPlate) |
                (Type::Psychic,     Item::TwistedSpoon | Item::MindPlate) |
                (Type::Fire,        Item::Charcoal | Item::FlamePlate) |
                (Type::Dragon,      Item::DragonFang | Item::DracoPlate) |
                (Type::Normal,      Item::SilkScarf) | 
                (Type::Fairy,       Item::FairyFeather | Item::PixiePlate)
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

        let Some(item) = attacker.held_item
        else {
            return;
        };

        if item == Item::LifeOrb {
            *damage *= 8;
            *damage /= 5;
        }
    }

    //    Damage=((2×Level5+2)×Power×AD50+2)×Targets×PB×Weather×GlaiveRush×Critical×random×STAB×Type×Burn×other×ZMove×TeraShield
    fn calc_damage(&self, used_move: &Move, attacker: &Pokemon, defender: &Pokemon, state: &BattleState) -> u16 {

        let attack_stat = self.get_attacking_stat(used_move, attacker, defender, state) as u32;
        let defense_stat = self.get_defending_stat(used_move, attacker, defender, state) as u32;

        let attacker_data = self.data_handler.get_pokemon_data(attacker.id);
        let defender_data = self.data_handler.get_pokemon_data(defender.id);

        let power = self.get_move_power(used_move, attacker, defender, state);

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

        // crit
        if rand::random_ratio(1, 16) {
            damage *= 2;
        }

        damage *= rand::random_range(85..=100);
        damage /= 100;

        // stab
        if attacker_data.is_type(used_move.move_type) {
            damage *= 3;
            damage /= 2;
        }

        damage *= self.data_handler.type_chart.get(used_move.move_type, defender_data.type1) as u32;
        damage /= 100;

        if let Some(type2) = defender_data.type2 {
            damage *= self.data_handler.type_chart.get(used_move.move_type, type2) as u32;
            damage /= 100;
        }

        // burn
        if used_move.class == MoveClass::Physical && attacker.status == Some(NonVolatileStatus::Burn) {
            damage /= 2;
        }

        // tons of random stuff
        self.apply_item_boosts(&mut damage, used_move, attacker);

        let mut damage_u16 = damage as u16;

        if used_move.has_effect(MoveEffect::FalseSwipe) && damage_u16 >= defender.hp {
            damage_u16 = defender.hp;
        }
        
        damage_u16
    
    }

    fn deal_damage(&self, side: &mut Side, mut damage: u16, used_move: &Move, state: &BattleState) {

        let defender = side.get_active_mut();

        let defender_ability = self.get_active_ability(&defender, state);

        if damage >= defender.hp {

            if defender.hp == defender.max_hp {
                if defender_ability == Ability::Sturdy {
                    damage = defender.hp - 1;
                }
                else if defender.held_item == Some(Item::FocusSash) {
                    damage = defender.hp - 1;
                    defender.held_item = None;
                }
            }

            if defender.held_item == Some(Item::FocusBand) && rand::random_ratio(1, 10) {
                damage = defender.hp - 1;
            }
        }

        defender.hp = defender.hp.saturating_sub(damage);

        if defender.hp == 0 {
            // faint stuff
        }

        if used_move.move_type == Type::Dark && defender_ability == Ability::Justified {
            side.apply_stat_changes(Stat::Attack, 1);
        }
    }

    fn perform_action(&self, action: BattleAction, using_side: &mut Side, other_side: &mut Side, state: &mut BattleState) {

        match action {
            BattleAction::Move(move_id) => {

                let used_move = self.data_handler.get_move(move_id);

                let mut damage = 0;

                if used_move.class != MoveClass::Status {
                    damage = self.calc_damage(used_move, &using_side.get_active_mut(), &other_side.get_active_mut(), &state);
                    self.deal_damage(other_side, damage, used_move, &state);
                }

                if let Some((effect, chance)) = used_move.effect {
                    if rand::random_ratio(chance as u32, 100) {
                        self.apply_effect_after_use(effect, using_side, other_side, damage);
                    }
                }
            }
            BattleAction::Switch(mon_idx) =>  {
                using_side.active_pokemon = mon_idx as usize;
            }
            BattleAction::Item(_) => todo!()
        }
    }

    fn apply_effect_after_use(&self, effect: MoveEffect, user_side: &mut Side, target_side: &mut Side, move_damage: u16) {

        let using_mon = user_side.get_active_mut();
        let target_mon = target_side.get_active_mut();

        use MoveEffect as ME;
        match effect {
            ME::Sleep => target_side.try_apply_status(NonVolatileStatus::Sleep),
            ME::PoisonChance => target_side.try_apply_status(NonVolatileStatus::Poison),
            ME::DrainHalf => using_mon.hp = using_mon.max_hp.min(using_mon.hp + move_damage / 2),
            ME::Burn | ME::BurnChance => target_side.try_apply_status(NonVolatileStatus::Burn),
            ME::FreezeChance => target_side.try_apply_status(NonVolatileStatus::Freeze),
            ME::Paralyze | ME::ParalyzeChance => target_side.try_apply_status(NonVolatileStatus::Paralysis),
            ME::FaintUser => using_mon.hp = 0,
            ME::DreamEater => todo!(),
            ME::UseTargetsLastMove => todo!(),
            ME::RaiseUserAttack1 => user_side.apply_stat_changes(Stat::Attack, 1),
            ME::RaiseUserDefense1 => user_side.apply_stat_changes(Stat::Defense, 1),
            ME::RaiseUserSpecialAttack1 => user_side.apply_stat_changes(Stat::SpecialAttack, 1),
            //ME::RaiseUserSpecialDefense1 => using_side.apply_stat_changes(Stat::SpecialDefense, 1),
            ME::RaiseUserSpeed1 => user_side.apply_stat_changes(Stat::Speed, 1),
            ME::NeverMiss => todo!(),
            ME::LowerTargetAttack1 => target_side.apply_stat_changes(Stat::Attack, -1),
            ME::LowerTargetDefense1 => target_side.apply_stat_changes(Stat::Defense, -1),
            // ME::LowerTargetSpecialAttack1 => using_side.apply_stat_changes(Stat::SpecialAttack, 1),
            // ME::LowerTargetSpecialDefense1 => using_side.apply_stat_changes(Stat::SpecialDefense, 1),
            ME::LowerTargetSpeed1 => target_side.apply_stat_changes(Stat::Speed, -1),
            ME::LowerTargetAccuracy1 => target_side.apply_stat_changes(Stat::Accuracy, -1),
            ME::LowerTargetEvasion1 => target_side.apply_stat_changes(Stat::Evasion, -1),
            ME::ResetTargetStats => target_side.reset_stat_changes(),
            ME::Bide => todo!(),
            ME::ForceSwitch => todo!(),
            ME::Hit2To5Times => todo!(),
            ME::Conversion => todo!(),
            ME::FlinchChance => todo!(),
            ME::HealUserHalf => todo!(),
            ME::BadlyPoison | ME::BadlyPoisonChance => target_side.try_apply_status(NonVolatileStatus::BadlyPoison),
            ME::ScatterMoney => todo!(),
            ME::LightScreen => todo!(),
            ME::TriAttack => todo!(),
            ME::Rest => todo!(),
            ME::OHKO => todo!(),
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
            ME::Confuse | ME::ConfuseAllTargets | ME::ConfuseChance => todo!(),
            ME::RaiseUserAttack2 => user_side.apply_stat_changes(Stat::Attack, 2),
            ME::RaiseUserDefense2 => user_side.apply_stat_changes(Stat::Defense, 2),
            ME::RaiseUserSpeed2 => user_side.apply_stat_changes(Stat::Speed, 2),
            ME::RaiseUserSpecialAttack2 => user_side.apply_stat_changes(Stat::SpecialAttack, 2),
            ME::RaiseUserSpecialDefense2 => user_side.apply_stat_changes(Stat::SpecialDefense, 2),
            ME::Transform => todo!(),
            ME::LowerTargetAttack2 => target_side.apply_stat_changes(Stat::Attack, 2),
            ME::LowerTargetDefense2 => target_side.apply_stat_changes(Stat::Defense, 2),
            ME::LowerTargetSpeed2 => target_side.apply_stat_changes(Stat::Speed, 2),
            ME::LowerTargetSpecialAttack2 => target_side.apply_stat_changes(Stat::SpecialAttack, 2),
            ME::LowerTargetSpecialDefense2 => target_side.apply_stat_changes(Stat::SpecialDefense, 2),
            ME::Reflect => todo!(),
            ME::Poison => target_side.try_apply_status(NonVolatileStatus::Poison),
            ME::LowerTargetAttack1Chance => target_side.apply_stat_changes(Stat::Attack, -1),
            ME::LowerTargetDefense1Chance => target_side.apply_stat_changes(Stat::Defense, -1),
            ME::LowerTargetSpeed1Chance => target_side.apply_stat_changes(Stat::Speed, -1),
            ME::LowerTargetSpecialAttack1Chance => target_side.apply_stat_changes(Stat::SpecialAttack, -1),
            ME::LowerTargetSpecialDefense1Chance => target_side.apply_stat_changes(Stat::SpecialDefense, -1),
            ME::LowerTargetAccuracy1Chance => target_side.apply_stat_changes(Stat::Accuracy, -1),
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
            ME::Sandstorm => todo!(),
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
            ME::RainDance => todo!(),
            ME::SunnyDay => todo!(),
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
            ME::Hail => todo!(),
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
            ME::RaiseUserAttackSpeed1 => todo!(),
            ME::Camouflage => todo!(),
            ME::Roost => todo!(),
            ME::Gravity => todo!(),
            ME::MiracleEye => todo!(),
            ME::WakeUpSlap => todo!(),
            ME::LowerUserSpeed1 => user_side.apply_stat_changes(Stat::Speed, -1),
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
            ME::GastroAcid => todo!(),
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
            ME::TypeBasedOnDriveOrPlate => todo!(),
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
                user_side.apply_stat_changes(Stat::SpecialAttack, 1);
                user_side.apply_stat_changes(Stat::SpecialDefense, 1);
                user_side.apply_stat_changes(Stat::Speed, 1);
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
                user_side.apply_stat_changes(Stat::Attack, 2);
                user_side.apply_stat_changes(Stat::SpecialAttack, 2);
                user_side.apply_stat_changes(Stat::Speed, 2);
                user_side.apply_stat_changes(Stat::Defense, -2);
                user_side.apply_stat_changes(Stat::SpecialDefense, -2);
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
            ME::TailGlow => user_side.apply_stat_changes(Stat::SpecialAttack, 3),
            ME::Coil => {
                user_side.apply_stat_changes(Stat::Attack, 1);
                user_side.apply_stat_changes(Stat::Defense, 1);
                user_side.apply_stat_changes(Stat::Accuracy, 1);
            },
            ME::Thief => todo!(),
            ME::WaterPledge => todo!(),
            ME::FirePledge => todo!(),
            ME::GrassPledge => todo!(),
            ME::WorkUp => {
                user_side.apply_stat_changes(Stat::Attack, 1);
                user_side.apply_stat_changes(Stat::SpecialAttack, 1);
            },
            ME::CottonGuard => user_side.apply_stat_changes(Stat::Defense, 3),
            ME::RelicSong => todo!(),
            ME::RockTomb => target_side.apply_stat_changes(Stat::Speed, -1),
            ME::FreezeShock => todo!(),
            ME::IceBurn => todo!(),
            ME::Hurricane => todo!(),
            ME::VCreate => {
                user_side.apply_stat_changes(Stat::Defense, -1);
                user_side.apply_stat_changes(Stat::SpecialDefense, -1);
                user_side.apply_stat_changes(Stat::Speed, -1);
            },
            ME::FusionFlare => todo!(),
            ME::FusionBolt => todo!(),
            ME::FlyingPress => todo!(),
            ME::Belch => todo!(),
            ME::Rototiller => todo!(),
            ME::StickyWeb => todo!(),
            ME::FellStinger => todo!(),
            _ => {}
        }
    }

    fn on_turn_end(&self) {
    }

    pub fn simulate_turn(&self, side1_action: BattleAction, side2_action: BattleAction, state: &mut BattleState) {
        
        let side1_pokemon = &state.side1.team[state.side1.active_pokemon];
        let side2_pokemon = &state.side2.team[state.side2.active_pokemon];

        let side1_priority = self.get_priority(&side1_action);
        let side2_priority = self.get_priority(&side2_action);

        let mut side1_first = rand::random::<bool>();

        let side1_speed = self.get_speed_stat(side1_pokemon, state);
        let side2_speed = self.get_speed_stat(side2_pokemon, state);

        if side1_priority != side2_priority {
            side1_first = side1_priority > side2_priority;
        }
        else if side1_speed != side2_speed {
            side1_first = side1_speed != side2_speed
        }

        if side1_first {
            self.perform_action(side1_action, &mut state.side1, &mut state.side2, state);
            self.perform_action(side2_action, &mut state.side2, &mut state.side1, state);
        }
        else {
            self.perform_action(side2_action, &mut state.side2, &mut state.side1, state);
            self.perform_action(side1_action, &mut state.side1, &mut state.side2, state);
        }

        self.on_turn_end();

        println!("{:#?}", state.side1);
        println!("{:#?}", state.side2);
        
    }
}