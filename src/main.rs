mod runescape;
use runescape::gear::item_group::ItemGroup;
use runescape::gear::item_group::group_similar_items;
use runescape::gear::GearKind;
use runescape::osrsbox_db::types::AttackStyle::Aggressive;
use runescape::graph::level::*;
use runescape::osrsbox_db;
use pathfinding::prelude::dijkstra;

#[macro_use]
extern crate itertools;

fn main() -> std::io::Result<()> {
	// let weapons = osrsbox_db::request(osrsbox_db::types::Slot::Weapon)?;

	// let scimitars = vec![
	// 	&weapons[&1323], // iron
	// 	&weapons[&1325], // steel
	// 	&weapons[&1327], // black
	// 	&weapons[&1329], // mith
	// 	&weapons[&1331], // addy
	// 	&weapons[&1333], // rune
	// 	&weapons[&4587], // dragon
	// 	&weapons[&13265], // abyssal dagger
	// ];

	let start = Melee::new(1, 1, 1);
	let goal = Melee::new(99, 99, 1);

	let result = dijkstra(&start, |p| p.successors(&Vec::new()), |p| *p == goal);

	// horrifically ugly
	if let Some((v, h)) = result {
		let mut attack = 1;
		let mut strength = 1;
		let mut attack_last = false;
		let mut last = 0;
		for s in v.windows(2) {
			match s[0].difference(&s[1]) {
				(0, 1) => {
					if attack_last {
						println!("train attack {} levels (attack: {}, strength: {})", last, attack, strength);
						last = 0;
						attack_last = false;
					}
					strength += 1;
				},
				(1, 0) => {
					if !attack_last {
						println!("train strength {} levels (attack: {}, strength: {})", last, attack, strength);
						last = 0;
						attack_last = true;
					}
					attack += 1;
				},
				_ => unreachable!(),
			}
			last += 1;
		}
		if attack_last {
			println!("train attack {} levels (attack: {}, strength: {})", last, attack, strength);
		} else {
			println!("train strength {} levels (attack: {}, strength: {})", last, attack, strength);
		}
		println!("total time: {} hours", h);
	}

	Ok(())
}

// fn main() -> std::io::Result<()> {

// }
