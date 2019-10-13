use crate::runescape::gear::item_group::filter_elided_items;
use crate::runescape::gear::GearCache;

use crate::runescape::gear::item_group::ItemGroup;
use crate::runescape::RunescapeInt;
use crate::runescape::osrsbox_db::types::*;
use std::hash::{Hash, Hasher};
use ordered_float::NotNan;

use itertools::Itertools;

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

// #[derive(Debug, Clone, Copy, PartialEq)]
// enum Direction {
// 	Attack,
// 	Strength,
// 	Defence,
// }

#[derive(Debug, Clone)]
pub struct Melee {
	attack: Level,
	strength: Level,
	defence: Level,
	pub gear_that_got_us_here: Option<Vec<ItemGroup>>,
}

impl Melee {
	pub fn new(attack: RunescapeInt, strength: RunescapeInt, defence: RunescapeInt, gear: Option<Vec<ItemGroup>>) -> Self {
		Self {
			attack: Level { value: attack },
			strength: Level { value: strength },
			defence: Level { value: defence },
			gear_that_got_us_here: gear,
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

	pub fn xp_per_hour(&self, style: AttackStyle, items: &Vec<ItemGroup>) -> f64 {
		let stats = sum_stats(items);
		let attack_speed = find_weapon_speed(items).expect("missing weapon");

		(self.dps(style, stats, attack_speed) * 4.0) * (60.0 * 60.0)
	}

	fn hours_to_level(&self, style: AttackStyle, items: &Vec<ItemGroup>) -> f64 {
		use AttackStyle::*;

		let xp = match style {
			Accurate   => self.attack.xp_to_next_level(),
			Aggressive => self.strength.xp_to_next_level(),
			Defensive  => self.defence.xp_to_next_level(),
			_          => unreachable!(),
		};

		(xp as f64) / self.xp_per_hour(style, items)
	}

	pub fn successors(&self, gear_cache: &GearCache, goal: &Self) -> Vec<(Self, NotNan<f64>)> {
		let mut v = Vec::with_capacity(3);
		if let Some(successor) = self.successor(AttackStyle::Accurate,   gear_cache, goal) { v.push(successor) }
		if let Some(successor) = self.successor(AttackStyle::Aggressive, gear_cache, goal) { v.push(successor) }
		if let Some(successor) = self.successor(AttackStyle::Defensive,  gear_cache, goal) { v.push(successor) }
		v
	}

	fn successor(&self, style: AttackStyle, gear_cache: &GearCache, goal: &Self) -> Option<(Self, NotNan<f64>)> {
		let mut attack = self.attack.value;
		let mut strength = self.strength.value;
		let mut defence = self.defence.value;

		let breakpoint = gear_cache.get_breakpoint(attack, strength, defence);

		match style {
			AttackStyle::Accurate   => { attack   += 1; if attack   > 127 || attack   > goal.attack.value   { return None } },
			AttackStyle::Aggressive => { strength += 1; if strength > 127 || strength > goal.strength.value { return None } },
			AttackStyle::Defensive  => { defence  += 1; if defence  > 127 || defence  > goal.defence.value  { return None } },
			_                       => unreachable!(),
		};

		let weapon = gear_cache.get_by_slot_full(Slot::Weapon, breakpoint, AttackType::Slash, style);
		let ammo = gear_cache.get_by_slot_full(Slot::Ammo, breakpoint, AttackType::Slash, style);
		let head = gear_cache.get_by_slot_full(Slot::Head, breakpoint, AttackType::Slash, style);
		let cape = gear_cache.get_by_slot_full(Slot::Cape, breakpoint, AttackType::Slash, style);
		let neck = gear_cache.get_by_slot_full(Slot::Neck, breakpoint, AttackType::Slash, style);
		let body = gear_cache.get_by_slot_full(Slot::Body, breakpoint, AttackType::Slash, style);
		let legs = gear_cache.get_by_slot_full(Slot::Legs, breakpoint, AttackType::Slash, style);
		let shield = gear_cache.get_by_slot_full(Slot::Shield, breakpoint, AttackType::Slash, style);
		let hands = gear_cache.get_by_slot_full(Slot::Hands, breakpoint, AttackType::Slash, style);
		let feet = gear_cache.get_by_slot_full(Slot::Feet, breakpoint, AttackType::Slash, style);
		let ring = gear_cache.get_by_slot_full(Slot::Ring, breakpoint, AttackType::Slash, style);

		let all = vec![
			filter_elided_items(&weapon),
			filter_elided_items(&ammo),
			filter_elided_items(&head),
			filter_elided_items(&cape),
			filter_elided_items(&neck),
			filter_elided_items(&body),
			filter_elided_items(&legs),
			filter_elided_items(&shield),
			filter_elided_items(&hands),
			filter_elided_items(&feet),
			filter_elided_items(&ring),
		];

		let mut max_hours = std::f64::INFINITY;
		let mut gear = None;
		for set in all.into_iter().multi_cartesian_product() {
			let hours = self.hours_to_level(style, &set);
			if hours < max_hours {
				max_hours = hours;
				gear.replace(set);
			}
		}

		if let None = gear {
			unreachable!("couldn't find gear");
		}

		Some((Self::new(attack, strength, defence, gear), NotNan::new(max_hours).unwrap()))
	}
}

fn sum_stats(items: &Vec<ItemGroup>) -> (RunescapeInt, RunescapeInt) {
	let mut attack_bonus = 0;
	let mut strength_bonus = 0;
	for item in items {
		attack_bonus += item.attack_value;
		strength_bonus += item.strength_value;
	}
	(attack_bonus, strength_bonus)
}

fn find_weapon_speed(items: &Vec<ItemGroup>) -> Option<RunescapeInt> {
	for item in items {
		if let Some(speed) = item.attack_speed {
			return Some(speed);
		}
	}
	None
}

impl PartialEq for Melee {
	fn eq(&self, other: &Self) -> bool {
		self.attack == other.attack &&
		self.strength == other.strength &&
		self.defence == other.defence
	}
}

impl Hash for Melee {
	fn hash<H: Hasher>(&self, state: &mut H) {
        self.attack.hash(state);
        self.strength.hash(state);
        self.defence.hash(state);
    }
}

impl std::fmt::Display for Melee {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "({}, {}, {})", self.attack.value, self.strength.value, self.defence.value)
	}
}

impl Eq for Melee {}
