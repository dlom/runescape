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
			inner_vec.into_iter().unique().collect::<Vec<RunescapeInt>>()
		}
	};
}
