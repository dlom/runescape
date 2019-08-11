use super::super::osrsbox_db;
use super::super::RunescapeInt;
use std::hash::{Hash, Hasher};
use ordered_float::NotNan;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Level {
	value: RunescapeInt,
}

type Stance = usize;
const ATTACK_STANCE:     Stance = 0;
const STRENGTH_STANCE:   Stance = 1;
// const CONTROLLED_STANCE: Stance = 2;
// const DEFENCE_STANCE:    Stance = 3;

const CRAB_MAX_DEFENCE_ROLL: RunescapeInt = (1 + 1 + 8) * (0 + 64);
const GAME_TICK: f64 = 0.6;

impl Level {
	pub fn xp_to_next_level(&self) -> u64 {
		match self.value {
			1..=98 => super::XP_TABLE[self.value as usize] - super::XP_TABLE[self.value as usize - 1],
			99     => 0,
			_      => 0xFFFFFFFFFFFFFFFF
		}
	}
}

#[derive(Debug, Clone)]
pub struct AttStrPair {
	attack: Level,
	strength: Level,
}

impl AttStrPair {
	pub fn new(attack: RunescapeInt, strength: RunescapeInt) -> Self {
		AttStrPair {
			attack: Level { value: attack },
			strength: Level { value: strength },
		}
	}

	fn best_weapon<'a>(&self, weapons: &'a Vec<&'a osrsbox_db::types::Item>) -> Option<&'a osrsbox_db::types::Item> {
		for ref item in weapons.iter().rev() {
			let reqs = &item.equipment.requirements;
			let reqs = reqs.as_ref().unwrap();
			if Some(self.attack.value) >= reqs.attack {
				return Some(item)
			}
		}
		None
	}

	fn effective_strength(&self, stance: Stance, weapon: &osrsbox_db::types::Item) -> RunescapeInt {
		use osrsbox_db::types::AttackStyle::*;
		let weapon = weapon.weapon.as_ref().unwrap();

		let strength_level    = self.strength.value as f64;
		let potion_effect     = 0.0;
		let prayer_multiplyer = 1.0;
		let style_bonus = match weapon.stances.get(stance).unwrap().attack_style {
			Some(Aggressive) => 3.0,
			Some(Controlled) => 1.0,
			_                => 0.0,
		};

		((strength_level + potion_effect) * prayer_multiplyer + style_bonus + 8.0) as RunescapeInt
	}

	fn effective_attack(&self, stance: Stance, weapon: &osrsbox_db::types::Item) -> RunescapeInt {
		use osrsbox_db::types::AttackStyle::*;
		let weapon = weapon.weapon.as_ref().unwrap();

		let attack_level      = self.attack.value as f64;
		let potion_effect     = 0.0;
		let prayer_multiplyer = 1.0;
		let style_bonus = match weapon.stances.get(stance).unwrap().attack_style {
			Some(Accurate)   => 3.0,
			Some(Controlled) => 1.0,
			_                => 0.0,
		};

		((attack_level + potion_effect) * prayer_multiplyer + style_bonus + 8.0) as RunescapeInt
	}

	fn max_hit(&self, stance: Stance, weapon: &osrsbox_db::types::Item) -> RunescapeInt {
		let base = 0.5;
		let effective_strength = self.effective_strength(stance, weapon) as f64;
		let bonus = weapon.equipment.melee_strength as f64;

		(base + effective_strength * (bonus + 64.0) / 640.0) as RunescapeInt
	}

	fn max_attack_roll(&self, stance: Stance, item: &osrsbox_db::types::Item) -> RunescapeInt {
		use osrsbox_db::types::AttackType::*;
		let weapon = item.weapon.as_ref().unwrap();

		let effective_attack = self.effective_attack(stance, item);
		let bonus = match weapon.stances.get(stance).unwrap().attack_type {
			Some(Stab)  => item.equipment.attack_stab,
			Some(Slash) => item.equipment.attack_slash,
			Some(Crush) => item.equipment.attack_crush,
			_           => 0,
		};

		effective_attack * (bonus + 64)
	}

	fn hit_chance(&self, stance: Stance, weapon: &osrsbox_db::types::Item, enemy_max_defence_roll: RunescapeInt) -> f64 {
		let max_attack_roll = self.max_attack_roll(stance, weapon) as f64;
		let max_defence_roll = enemy_max_defence_roll as f64;
		if max_attack_roll > max_defence_roll {
			1.0 - (max_defence_roll + 2.0) / (2.0 * (max_attack_roll + 1.0))
		} else {
			max_attack_roll / (2.0 * max_defence_roll + 1.0)
		}
	}

	pub fn dps(&self, stance: Stance, item: &osrsbox_db::types::Item) -> f64 {
		let weapon = item.weapon.as_ref().unwrap();

		let max_hit = self.max_hit(stance, item) as f64;
		let hit_chance = self.hit_chance(stance, item, CRAB_MAX_DEFENCE_ROLL);
		let attack_interval = weapon.attack_speed as f64;
		hit_chance * (max_hit / 2.0) / (attack_interval * GAME_TICK)
	}

	fn xp_per_hour(&self, stance: Stance, weapon: &osrsbox_db::types::Item) -> f64 {
		(self.dps(stance, weapon) * 4.0) * (60.0 * 60.0)
	}

	pub fn hours_to_level(&self, direction: Stance, weapon: &osrsbox_db::types::Item) -> f64 {
		let xp = match direction {
			ATTACK_STANCE => self.attack.xp_to_next_level(),
			STRENGTH_STANCE => self.strength.xp_to_next_level(),
			_ => unreachable!(),
		} as f64;
		xp / self.xp_per_hour(direction, weapon)
	}

	pub fn successors(&self, weapons: &Vec<&osrsbox_db::types::Item>) -> Vec<(Self, NotNan<f64>)> {
		let weapon = self.best_weapon(weapons).unwrap();
		// println!("{}, {}: {}", self.attack.value, self.strength.value, weapon.name);
		vec![
			(Self::new(self.attack.value + 0, self.strength.value + 1), NotNan::new(self.hours_to_level(STRENGTH_STANCE, weapon)).unwrap()),
			(Self::new(self.attack.value + 1, self.strength.value + 0), NotNan::new(self.hours_to_level(ATTACK_STANCE, weapon)).unwrap()),
		]
	}

	pub fn difference(&self, other: &Self) -> (RunescapeInt, RunescapeInt) {
		(other.attack.value - self.attack.value, other.strength.value - self.strength.value)
	}
}

impl PartialEq for AttStrPair {
	fn eq(&self, other: &Self) -> bool {
		self.attack == other.attack && self.strength == other.strength
	}
}

impl Hash for AttStrPair {
	fn hash<H: Hasher>(&self, state: &mut H) {
        self.attack.hash(state);
        self.strength.hash(state);
    }
}

impl Eq for AttStrPair {}
