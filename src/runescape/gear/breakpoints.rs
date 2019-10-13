use crate::runescape::graph::xp_to_level;
use crate::runescape::graph::level_to_xp;
use crate::runescape::osrsbox_db::types::Stats;
use crate::runescape::RunescapeInt;

pub type Breakpoint = (RunescapeInt, RunescapeInt, RunescapeInt);

impl From<Breakpoint> for Stats {
	fn from(breakpoint: Breakpoint) -> Self {
		let total_xp = level_to_xp(breakpoint.0) + level_to_xp(breakpoint.1) + level_to_xp(breakpoint.2);
		let hp_xp = total_xp / 3;
		let hp_level = xp_to_level(hp_xp);
		Stats {
			attack:    Some(breakpoint.0),
			strength:  Some(breakpoint.1),
			defence:   Some(breakpoint.2),
			hitpoints: Some(hp_level),
			prayer:    None,
			ranged:    None,
			magic:     None,
		}
	}
}

macro_rules! breakpoints {
	($iter:expr, $prop:ident) => {
		{
			let mut inner_vec = Vec::new();
			for (_, item) in $iter {
				if let Some(reqs) = &item.equipment.requirements {
					if let Some(value) = reqs.$prop {
						inner_vec.push(value);
					}
				}
			}
			inner_vec.sort();
			inner_vec.into_iter().unique().collect::<Vec<RunescapeInt>>()
		}
	};
}

// #[derive(Debug)]
// pub struct BreakpointCache<'a> {
// 	cache: BTreeMap<(Slot, Breakpoint), Vec<ItemGroup<'a>>>,
// }

// impl<'a> BreakpointCache<'a> {
// 	pub fn new() -> Self {
// 		Self {
// 			cache: BTreeMap::new(),
// 		}
// 	}

// 	pub fn get(&mut self, slot: Slot, breakpoint: Breakpoint, gear_cache: &'a GearCache) -> &Vec<ItemGroup> {
// 		self.cache.entry((slot, breakpoint)).or_insert_with(|| {
// 			let slot_gear = gear_cache.get_slot_gear(slot);
// 			item_group::group_similar_items(slot_gear)
// 		})
// 	}
// }
