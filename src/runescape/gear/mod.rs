use breakpoints::Breakpoint;
use std::collections::BTreeMap;

use crate::runescape::RunescapeInt;
use crate::runescape::osrsbox_db::types::*;
use crate::runescape::osrsbox_db;

use itertools::Itertools;


#[macro_use]
pub mod breakpoints;
pub mod item_group;

#[derive(Debug)]
pub struct GearCache {
	gear: BTreeMap<RunescapeInt, Item>,
	attack_breakpoints: Vec<RunescapeInt>,
	strength_breakpoints: Vec<RunescapeInt>,
	defence_breakpoints: Vec<RunescapeInt>,
}

pub enum GearKind {
	Melee,
	Ranged,
	Magic,
}

impl GearCache {
	pub fn new(kind: GearKind) -> std::io::Result<Self> {
		let predicate = match kind {
			GearKind::Melee => is_melee_gear,
			_               => unreachable!(),
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

		Ok(GearCache {
			attack_breakpoints:   breakpoints!(&gear, attack),
			strength_breakpoints: breakpoints!(&gear, strength),
			defence_breakpoints:  breakpoints!(&gear, defence),
			gear: normalize_gear(gear, predicate),
		})
	}

	pub fn get_slot_gear<'a>(&'a self, slot: Slot, id_blacklist: &Vec<RunescapeInt>) -> Vec<&'a Item> {
		let mut v = Vec::new();
		for (_, item) in &self.gear {
			if item.equipment.slot == slot && !id_blacklist.contains(&item.id) {
				v.push(item)
			}
		}
		v
	}

	pub fn get_breakpoint(&self, attack: RunescapeInt, strength: RunescapeInt, defence: RunescapeInt) -> Breakpoint {
		// let attack_breakpoint = self.attack_breakpoint.filter
		unimplemented!()
	}

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
