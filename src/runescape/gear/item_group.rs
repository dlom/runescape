use std::iter::IntoIterator;

use crate::runescape::RunescapeInt;
use crate::runescape::osrsbox_db::types::*;

use itertools::Itertools;

#[derive(Clone, Debug)]
pub struct ItemGroup<'a> {
	pub equipment: Equipment,
	pub items: Vec<&'a Item>
}

fn melee_stat_keyer(item: &&Item) -> (RunescapeInt, RunescapeInt, RunescapeInt, RunescapeInt, Slot) {
	(item.equipment.attack_stab, item.equipment.attack_slash, item.equipment.attack_crush, item.equipment.melee_strength, item.equipment.slot)
}

pub fn group_similar_items<'a, T: IntoIterator<Item=&'a Item>>(iter: T) -> Vec<ItemGroup<'a>> {
	iter.into_iter()
		.sorted_by(|a, b| {
			let a = (a.equipment.melee_strength, a.equipment.attack_stab, a.equipment.attack_slash, a.equipment.attack_crush);
			let b = (b.equipment.melee_strength, b.equipment.attack_stab, b.equipment.attack_slash, b.equipment.attack_crush);
			Ord::cmp(&a, &b)
		})
		.group_by(melee_stat_keyer)
		.into_iter()
		.map(|(key, group)| {
			let items: Vec<&'a Item> = group.fold(Vec::new(), |mut v, item| {
				v.push(item);
				v
			});
			let equipment = Equipment {
				attack_stab:     key.0,
				attack_slash:    key.1,
				attack_crush:    key.2,
				attack_magic:    0,
				attack_ranged:   0,
				defence_stab:    0,
				defence_slash:   0,
				defence_crush:   0,
				defence_magic:   0,
				defence_ranged:  0,
				melee_strength:  key.3,
				ranged_strength: 0,
				magic_damage:    0,
				prayer:          0,
				slot:            key.4,
				requirements:    None,
			};
			ItemGroup { equipment, items }
		})
		.collect()
}

impl ItemGroup<'_> {
	pub fn group_name(&self) -> String {
		if self.items.len() == 0 {
			return "Nothing".into();
		}

		let lowest_id_item = self.items.iter().fold(None, |o, item| {
			match o {
				None => Some(item),
				Some(other) => {
					if item.id < other.id {
						Some(item)
					} else {
						Some(other)
					}
				}
			}
		}).unwrap();

		format!("{}-like group", lowest_id_item.name)
	}
}

pub fn find_weapon<'a>(items: &'a Vec<&'a ItemGroup>) -> Option<&'a Weapon> {
	let group = items.iter().fold(None, |o, group| {
		match o {
			None => match group.equipment.slot {
				Slot::Weapon => Some(group),
				_      => None,
			}
			Some(other) => Some(other),
		}
	})?;

	group.items[0].weapon.as_ref()
}

pub fn sum_stats(items: &Vec<&ItemGroup>, style: AttackStyle) -> (RunescapeInt, RunescapeInt) { // attack bonus, melee strength
	use super::super::osrsbox_db::types::AttackType::*;
	let mut attack = 0;
	let mut strength = 0;

	let stances = &find_weapon(items).expect("missing weapon").stances;

	for item in items {
		let bonus = match style_to_type(&stances, style) {
			Some(Stab)  => item.equipment.attack_stab,
			Some(Slash) => item.equipment.attack_slash,
			Some(Crush) => item.equipment.attack_crush,
			_           => 0,
		};
		attack += bonus;
		strength += item.equipment.melee_strength;
	}

	(attack, strength)
}

fn style_to_type(stances: &Vec<Stance>, style: AttackStyle) -> Option<AttackType> {
	match stances.iter().find(|stance| stance.attack_style == Some(style)) {
		Some(stance) => stance.attack_type,
		None => None,
	}
}
