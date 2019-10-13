use std::collections::BTreeMap;
use std::iter::IntoIterator;

use crate::runescape::RunescapeInt;
use crate::runescape::gear::GearCache;
use crate::runescape::osrsbox_db::types::*;

use itertools::Itertools;

#[derive(Clone, Debug)]
pub struct ItemGroup {
	pub item_ids: Vec<RunescapeInt>,
	pub attack_value: RunescapeInt,
	pub strength_value: RunescapeInt,
	pub attack_type: AttackType,
	pub attack_style: Option<AttackStyle>,
	pub attack_speed: Option<RunescapeInt>,
}

#[derive(Debug)]
struct DecomposedItem {
	id: RunescapeInt,
	attack_value: RunescapeInt,
	strength_value: RunescapeInt,
	attack_type: AttackType,
	slot: Slot,
	attack_style: Option<AttackStyle>,
	attack_speed: Option<RunescapeInt>,
}

fn melee_stat_keyer(item: &DecomposedItem) -> (RunescapeInt, RunescapeInt, AttackType, Slot, Option<AttackStyle>, Option<RunescapeInt>) {
	(item.attack_value, item.strength_value, item.attack_type, item.slot, item.attack_style, item.attack_speed)
}

pub fn group_similar_items<'a, T: IntoIterator<Item=&'a Item>>(iter: T) -> Vec<ItemGroup> {
	iter.into_iter()
		.flat_map(decompose_item)
		.filter(|item| {
			item.attack_value >= 0 && item.strength_value >= 0
		})
		.filter(|item| {
			match item.attack_style {
				None => true,
				Some(style) => match style {
					AttackStyle::Accurate | AttackStyle::Aggressive | AttackStyle::Defensive => true,
					_ => false,
				}
			}
		})
		.sorted_by(|a, b| {
			let a = (a.attack_value, a.strength_value, a.attack_type, a.slot, a.attack_style, a.attack_speed);
			let b = (b.attack_value, b.strength_value, b.attack_type, b.slot, b.attack_style, b.attack_speed);
			Ord::cmp(&a, &b)
		})
		.group_by(melee_stat_keyer)
		.into_iter()
		.map(|(key, group)| {
			let item_ids = group.into_iter().map(|item| item.id).collect();
			ItemGroup {
				item_ids,
				attack_value: key.0,
				strength_value: key.1,
				attack_type: key.2,
				attack_style: key.4,
				attack_speed: key.5,
			}
		})
		.collect()
}

impl ItemGroup {
	pub fn empty_group(attack_type: AttackType) -> Self {
		ItemGroup {
			item_ids: Vec::new(),
			attack_value: 0,
			strength_value: 0,
			attack_type: attack_type,
			attack_style: None,
			attack_speed: None,
		}
	}

	pub fn group_identifier(&self) -> Option<RunescapeInt> {
		self.item_ids.iter().fold(None, |o, id| {
			match o {
				None => Some(*id),
				Some(other_id) => {
					if id < &other_id {
						Some(*id)
					} else {
						Some(other_id)
					}
				}
			}
		})
	}

	pub fn group_name(&self, gear_cache: &GearCache) -> String {
		match self.group_identifier() {
			None => "Nothing".into(),
			Some(id) => format!("{}-like group", gear_cache.get_by_id(id).unwrap().name),
		}
	}
}

// pub fn find_attack_style(items: &Vec<&ItemGroup>) -> Option<AttackStyle> {
// 	items.iter().fold(None, |o, group| {
// 		match o {
// 			None        => group.attack_style,
// 			Some(other) => Some(other),
// 		}
// 	})
// }

// fn style_to_type(stances: &Vec<Stance>, style: AttackStyle) -> Option<AttackType> {
// 	match stances.iter().find(|stance| stance.attack_style == Some(style)) {
// 		Some(stance) => stance.attack_type,
// 		None => None,
// 	}
// }

fn has_type_and_style(stances: &Vec<Stance>, attack_type: AttackType, attack_style: AttackStyle) -> bool {
	match stances.iter().find(|stance| stance.attack_type == Some(attack_type) && stance.attack_style == Some(attack_style)) {
		Some(_) => true,
		None => false,
	}
}

fn decompose_item_by_type_and_style(item: &Item, attack_type: AttackType, attack_style: AttackStyle) -> Option<DecomposedItem> {
	let id = item.id;
	let strength_value = item.equipment.melee_strength;
	let slot = item.equipment.slot;
	let (attack_speed, valid) = item.weapon.as_ref().map_or((None, None), |weapon| {
		(Some(weapon.attack_speed), Some(has_type_and_style(&weapon.stances, attack_type, attack_style)))
	});
	if let Some(_) = attack_speed {
		if let Some(false) = valid {
			return None;
		}
	}
	Some(DecomposedItem {
		id, strength_value, slot, attack_type,
		attack_style: match item.equipment.slot {
			Slot::Weapon => Some(attack_style),
			_            => None,
		},
		attack_speed: attack_speed,
		attack_value: match attack_type {
			AttackType::Stab => item.equipment.attack_stab,
			AttackType::Slash => item.equipment.attack_slash,
			AttackType::Crush => item.equipment.attack_crush,
			_ => return None,
		},
	})
}

fn decompose_item(item: &Item) -> Vec<DecomposedItem> {
	vec![
		decompose_item_by_type_and_style(item, AttackType::Stab, AttackStyle::Accurate),
		decompose_item_by_type_and_style(item, AttackType::Slash, AttackStyle::Accurate),
		decompose_item_by_type_and_style(item, AttackType::Crush, AttackStyle::Accurate),
		decompose_item_by_type_and_style(item, AttackType::Stab, AttackStyle::Aggressive),
		decompose_item_by_type_and_style(item, AttackType::Slash, AttackStyle::Aggressive),
		decompose_item_by_type_and_style(item, AttackType::Crush, AttackStyle::Aggressive),
		decompose_item_by_type_and_style(item, AttackType::Stab, AttackStyle::Defensive),
		decompose_item_by_type_and_style(item, AttackType::Slash, AttackStyle::Defensive),
		decompose_item_by_type_and_style(item, AttackType::Crush, AttackStyle::Defensive),
	].into_iter().flatten().collect()
}

pub fn filter_elided_items(items: &Vec<ItemGroup>) -> Vec<ItemGroup> {
	let mut attack_speeds: BTreeMap<RunescapeInt, Vec<ItemGroup>> = BTreeMap::new();
	let sorted = items.into_iter().sorted_by(|a, b| {
		Ord::cmp(&(a.attack_type, a.attack_value, a.strength_value), &(b.attack_type, b.attack_value, b.strength_value))
	}).rev();

	for item in sorted {
		let item_speed = match item.attack_speed {
			Some(attack_speed) => attack_speed,
			None => 1,
		};

		let mut added = false;
		let speed_vec = attack_speeds.entry(item_speed).or_insert_with(|| {
			added = true;
			vec![item.clone()]
		});
		if !added {
			for best in &mut speed_vec.into_iter() {
				if best.attack_value >= item.attack_value && best.strength_value >= item.strength_value {
					added = true;
				} else if best.attack_value <= item.attack_value && best.strength_value <= item.strength_value {
					std::mem::replace(best, item.clone());
					added = true;
				}
			}
			if !added {
				speed_vec.push(item.clone());
			}
		}
	}

	let mut out = Vec::new();
	for (_, mut v) in attack_speeds {
		out.append(&mut v);
	}
	out
}
