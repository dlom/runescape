use serde::{Serialize, Deserialize};
use super::super::RunescapeInt;

#[derive(Deserialize, Debug)]
pub struct Item {
	pub id:        RunescapeInt,
	pub name:      String,
	pub equipment: Equipment,
	pub weapon:    Option<Weapon>,
}

#[derive(Deserialize, Debug)]
pub struct Equipment {
	pub attack_stab:     RunescapeInt,
	pub attack_slash:    RunescapeInt,
	pub attack_crush:    RunescapeInt,
	pub attack_magic:    RunescapeInt,
	pub attack_ranged:   RunescapeInt,
	pub defence_stab:    RunescapeInt,
	pub defence_slash:   RunescapeInt,
	pub defence_crush:   RunescapeInt,
	pub defence_magic:   RunescapeInt,
	pub defence_ranged:  RunescapeInt,
	pub melee_strength:  RunescapeInt,
	pub ranged_strength: RunescapeInt,
	pub magic_damage:    RunescapeInt,
	pub prayer:          RunescapeInt,
	pub slot:            Slot,
	pub requirements:    Option<Stats>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Slot {
	#[serde(rename = "2h")]
	TwoH,
	Ammo,
	Body,
	Cape,
	Feet,
	Hands,
	Head,
	Legs,
	Neck,
	Ring,
	Shield,
	Weapon,
}

#[derive(Deserialize, Debug)]
pub struct Stats {
	pub attack:    Option<RunescapeInt>,
	pub strength:  Option<RunescapeInt>,
	pub defence:   Option<RunescapeInt>,
	pub hitpoints: Option<RunescapeInt>,
	pub prayer:    Option<RunescapeInt>,
	pub ranged:    Option<RunescapeInt>,
	pub magic:     Option<RunescapeInt>,
}

#[derive(Deserialize, Debug)]
pub struct Weapon {
	pub attack_speed: RunescapeInt,
	pub weapon_type:  String,
	pub stances:      Vec<Stance>,
}

#[derive(Deserialize, Debug)]
pub struct Stance {
	pub combat_style: String,
	pub attack_type:  Option<AttackType>,
	pub attack_style: Option<AttackStyle>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AttackType {
	Crush,
	#[serde(rename = "defensive casting")]
	DefensiveCasting,
	Slash,
	Spellcasting,
	Stab,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AttackStyle {
	Accurate,
	Aggressive,
	Controlled,
	Defensive,
	Magic,
}
