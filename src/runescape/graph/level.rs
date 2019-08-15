use crate::runescape::gear::item_group::find_weapon;
use crate::runescape::gear::item_group::sum_stats;
use crate::runescape::gear::item_group::ItemGroup;
use crate::runescape::RunescapeInt;
use crate::runescape::osrsbox_db::types::*;
use std::hash::{Hash, Hasher};
use ordered_float::NotNan;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Level {
	value: RunescapeInt,
}

const CRAB_MAX_DEFENCE_ROLL: RunescapeInt = (1 + 1 + 8) * (0 + 64);
const GAME_TICK: f64 = 0.6;

impl Level {
	pub fn xp_to_next_level(&self) -> u64 {
		match self.value {
			1..=126 => super::XP_TABLE[self.value as usize] - super::XP_TABLE[self.value as usize - 1],
			127     => 0,
			_       => 0xFFFFFFFFFFFFFFFF
		}
	}
}

enum Direction {
	Attack,
	Strength,
	Defence,
}

#[derive(Debug, Clone)]
pub struct Melee {
	attack: Level,
	strength: Level,
	defence: Level,
}

impl Melee {
	pub fn new(attack: RunescapeInt, strength: RunescapeInt, defence: RunescapeInt) -> Self {
		Melee {
			attack: Level { value: attack },
			strength: Level { value: strength },
			defence: Level { value: defence },
		}
	}

	fn effective_strength(&self, style: AttackStyle) -> RunescapeInt {
		use AttackStyle::*;

		let strength_level    = self.strength.value.min(99) as f64;
		let potion_effect     = 0.0;
		let prayer_multiplyer = 1.0;
		let style_bonus = match style {
			Aggressive => 3.0,
			Controlled => 1.0,
			_          => 0.0,
		};

		((strength_level + potion_effect) * prayer_multiplyer + style_bonus + 8.0) as RunescapeInt
	}

	fn effective_attack(&self, style: AttackStyle) -> RunescapeInt {
		use AttackStyle::*;

		let attack_level      = self.attack.value.min(99) as f64;
		let potion_effect     = 0.0;
		let prayer_multiplyer = 1.0;
		let style_bonus = match style {
			Accurate   => 3.0,
			Controlled => 1.0,
			_          => 0.0,
		};

		((attack_level + potion_effect) * prayer_multiplyer + style_bonus + 8.0) as RunescapeInt
	}

	fn max_hit(&self, style: AttackStyle, bonus: RunescapeInt) -> RunescapeInt {
		let base = 0.5;
		let effective_strength = self.effective_strength(style) as f64;
		let bonus = bonus as f64;

		(base + effective_strength * (bonus + 64.0) / 640.0) as RunescapeInt
	}

	fn max_attack_roll(&self, style: AttackStyle, bonus: RunescapeInt) -> RunescapeInt {
		let effective_attack = self.effective_attack(style);
		effective_attack * (bonus + 64)
	}

	fn hit_chance(&self, style: AttackStyle, attack_bonus: RunescapeInt, enemy_max_defence_roll: RunescapeInt) -> f64 {
		let max_attack_roll = self.max_attack_roll(style, attack_bonus) as f64;
		let max_defence_roll = enemy_max_defence_roll as f64;
		if max_attack_roll > max_defence_roll {
			1.0 - (max_defence_roll + 2.0) / (2.0 * (max_attack_roll + 1.0))
		} else {
			max_attack_roll / (2.0 * max_defence_roll + 1.0)
		}
	}

	pub fn dps(&self, style: AttackStyle, (attack_bonus, strength_bonus): (RunescapeInt, RunescapeInt), attack_speed: RunescapeInt) -> f64 {
		let max_hit = self.max_hit(style, strength_bonus) as f64;
		let hit_chance = self.hit_chance(style, attack_bonus, CRAB_MAX_DEFENCE_ROLL);
		let attack_interval = attack_speed as f64;
		hit_chance * (max_hit / 2.0) / (attack_interval * GAME_TICK)
	}

	pub fn xp_per_hour(&self, style: AttackStyle, items: &Vec<&ItemGroup>) -> f64 {
		let stats = sum_stats(items, style);
		let attack_speed = find_weapon(items).expect("missing weapon").attack_speed;

		(self.dps(style, stats, attack_speed) * 4.0) * (60.0 * 60.0)
	}

	fn hours_to_level(&self, direction: Direction, items: &Vec<&ItemGroup>) -> f64 {
		use AttackStyle::*;

		let (xp, style) = match direction {
			Direction::Attack   => (self.attack.xp_to_next_level(),   Accurate),
			Direction::Strength => (self.strength.xp_to_next_level(), Aggressive),
			Direction::Defence  => (self.defence.xp_to_next_level(),  Defensive),
		};

		(xp as f64) / self.xp_per_hour(style, items)
	}

	pub fn successors(&self, items: &Vec<&ItemGroup>) -> Vec<(Self, NotNan<f64>)> {
		let attack = self.attack.value;
		let strength = self.strength.value;
		let defence = self.defence.value;
		vec![
			(Self::new(attack + 1, strength + 0, defence + 0), NotNan::new(self.hours_to_level(Direction::Attack,   items)).unwrap()),
			(Self::new(attack + 0, strength + 1, defence + 0), NotNan::new(self.hours_to_level(Direction::Strength, items)).unwrap()),
			(Self::new(attack + 0, strength + 0, defence + 1), NotNan::new(self.hours_to_level(Direction::Defence,  items)).unwrap()),
		]
	}

	pub fn difference(&self, other: &Self) -> (RunescapeInt, RunescapeInt) {
		(other.attack.value - self.attack.value, other.strength.value - self.strength.value)
	}
}

impl PartialEq for Melee {
	fn eq(&self, other: &Self) -> bool {
		self.attack == other.attack && self.strength == other.strength
	}
}

impl Hash for Melee {
	fn hash<H: Hasher>(&self, state: &mut H) {
        self.attack.hash(state);
        self.strength.hash(state);
    }
}

impl Eq for Melee {}
