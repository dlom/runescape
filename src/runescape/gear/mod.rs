use std::rc::Rc;
use std::cell::RefCell;

use item_group::ItemGroup;
use breakpoints::Breakpoint;
use std::collections::BTreeMap;

use crate::runescape::RunescapeInt;
use crate::runescape::osrsbox_db::types::*;
use crate::runescape::osrsbox_db;

use itertools::Itertools;


#[macro_use]
pub mod breakpoints;
pub mod item_group;

pub struct GearCache {
	gear: BTreeMap<RunescapeInt, Item>,
	attack_breakpoints: Vec<RunescapeInt>,
	strength_breakpoints: Vec<RunescapeInt>,
	defence_breakpoints: Vec<RunescapeInt>,
	breakpoint_cache: RefCell<BTreeMap<(Slot, Breakpoint), BTreeMap<(AttackType, Option<AttackStyle>), Rc<Vec<ItemGroup>>>>>,
}

pub enum GearKind {
	Melee,
	// Ranged,
	// Magic,
}

impl GearCache {
	pub fn new(kind: GearKind) -> std::io::Result<Self> {
		let predicate = match kind {
			GearKind::Melee => is_melee_gear,
			// _               => unreachable!(),
		};

		let mut gear = BTreeMap::new();

		gear.append(&mut osrsbox_db::request(Slot::Ammo)?);
		gear.append(&mut osrsbox_db::request(Slot::Body)?);
		gear.append(&mut osrsbox_db::request(Slot::Cape)?);
		gear.append(&mut osrsbox_db::request(Slot::Feet)?);
		gear.append(&mut osrsbox_db::request(Slot::Hands)?);
		gear.append(&mut osrsbox_db::request(Slot::Head)?);
		gear.append(&mut osrsbox_db::request(Slot::Legs)?);
		gear.append(&mut osrsbox_db::request(Slot::Neck)?);
		gear.append(&mut osrsbox_db::request(Slot::Ring)?);
		gear.append(&mut osrsbox_db::request(Slot::Shield)?);
		gear.append(&mut osrsbox_db::request(Slot::Weapon)?);

		Ok(Self {
			attack_breakpoints:   breakpoints!(&gear, attack),
			strength_breakpoints: breakpoints!(&gear, strength),
			defence_breakpoints:  breakpoints!(&gear, defence),
			gear: normalize_gear(gear, predicate),
			breakpoint_cache: RefCell::new(BTreeMap::new()),
		})
	}

	pub fn get_by_id(&self, id: RunescapeInt) -> Option<&Item> {
		self.gear.get(&id)
	}

	fn get_by_slot(&self, slot: Slot) -> Vec<&Item> {
		let mut v = Vec::new();
		for (_, item) in &self.gear {
			if item.equipment.slot == slot {
				v.push(item)
			}
		}
		v
	}

	pub fn get_breakpoint(&self, attack: RunescapeInt, strength: RunescapeInt, defence: RunescapeInt) -> Breakpoint {
		let attack = check_breakpoint(&self.attack_breakpoints, attack);
		let strength = check_breakpoint(&self.strength_breakpoints, strength);
		let defence = check_breakpoint(&self.defence_breakpoints, defence);
		(attack, strength, defence)
	}

	pub fn get_by_slot_full(&self, slot: Slot, breakpoint: Breakpoint, attack_type: AttackType, attack_style: AttackStyle) -> Rc<Vec<ItemGroup>> {
		let attack_style = match slot {
			Slot::Weapon => Some(attack_style),
			_            => None,
		};
		Rc::clone(self.breakpoint_cache.borrow_mut().entry((slot, breakpoint)).or_insert_with(|| {
			let slot_gear = self.get_by_slot(slot);
			let filtered_gear = filter_by_breakpoint(slot_gear, breakpoint);
			let groups = item_group::group_similar_items(filtered_gear);
			let mut map = BTreeMap::new();
			for group in groups {
				map.entry((group.attack_type, group.attack_style)).or_insert(Vec::new()).push(group);
			}
			let mut rc_map = BTreeMap::new();
			for (key, value) in map.into_iter() {
				rc_map.insert(key, Rc::new(value));
			}
			rc_map
		}).get(&(attack_type, attack_style)).unwrap_or(&Rc::new(vec![ItemGroup::empty_group(attack_type)])))
	}
}

fn check_breakpoint(breakpoints: &Vec<RunescapeInt>, value: RunescapeInt) -> RunescapeInt {
	for pair in breakpoints.windows(2) {
		if value < pair[1] {
			return pair[0];
		}
	}
	*breakpoints.last().unwrap_or(&1)
}

fn normalize_gear<T: IntoIterator<Item=(RunescapeInt, Item)>, P: FnMut(&(RunescapeInt, Item)) -> bool>(iter: T, predicate: P) -> BTreeMap<RunescapeInt, Item> {
	iter.into_iter()
		.filter(predicate)
		.sorted_by(|(_, a), (_, b)| Ord::cmp(&a.name, &b.name))
		.group_by(|(_, item)| (item.name.clone(), item.equipment.clone()))
		.into_iter()
		.map(|(_, group)| {
			group.fold(None, |o, (id, item)| {
				match o {
					None => Some((id, item)),
					Some((other_id, other_item)) => {
						if id < other_id {
							Some((id, item))
						} else {
							Some((other_id, other_item))
						}
					}
				}
			}).unwrap()
		})
		.collect()
}

fn is_melee_gear((_, item): &(RunescapeInt, Item)) -> bool {
	if item.equipment.attack_stab > 0 {
		return true
	}
	if item.equipment.attack_slash > 0 {
		return true
	}
	if item.equipment.attack_crush > 0 {
		return true
	}
	if item.equipment.melee_strength > 0 {
		return true
	}
	false
}

fn filter_by_breakpoint<'a, T: IntoIterator<Item=&'a Item>>(iter: T, breakpoint: Breakpoint) -> Vec<&'a Item> {
	let stats = breakpoint.into();
	iter.into_iter()
		.filter(|item| {
			match &item.equipment.requirements {
				Some(requirements) => {
					requirements.has_requirements(&stats)
				},
				None => {
					match &item.weapon {
						Some(_) => false,
						None => true,
					}
				}
			}
		})
		.into_iter()
		.collect()
}
