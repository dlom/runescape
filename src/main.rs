mod runescape;
use runescape::graph::level::*;
use runescape::gear::GearCache;
use runescape::gear::GearKind;

use pathfinding::prelude::dijkstra;

fn main() -> std::io::Result<()> {
	let gear_cache = GearCache::new(GearKind::Melee)?;
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
	let goal = Melee::new(99, 99, 99);

	let _result = dijkstra(&start, |p| p.successors(&gear_cache), |p| *p == goal);

	Ok(())
}
