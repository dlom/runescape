mod runescape;

use pathfinding::prelude::dijkstra;

use runescape::gear::GearCache;
use runescape::gear::GearKind;
use runescape::graph::level::Melee;

fn main() -> std::io::Result<()> {
	println!("building gear cache");
	let gear_cache = GearCache::new(GearKind::Melee)?;
	println!("done");

	// let breakpoint = gear_cache.get_breakpoint(70, 70, 60);
	// dbg!(breakpoint);

	// return Ok(());

	let start = Melee::new(40, 40, 40, None);
	let goal = Melee::new(70, 70, 70, None);

	let result = dijkstra(&start, |p| p.successors(&gear_cache, &goal), |p| *p == goal);

	if let Some((v, h)) = result {
		for s in v.windows(2) {
			println!("train from {} to {} wearing:", s[0], s[1]);
			let got = (&s[1]).gear_that_got_us_here.clone().unwrap();
			for gear in got {
				println!("\t{}", gear.group_name(&gear_cache));
			}
		}
		println!("total time: {} hours", h);
	}

	Ok(())
}
