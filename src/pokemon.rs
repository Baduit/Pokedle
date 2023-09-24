use std::fs::{self, DirEntry};
use fs::File;
use std::path::{Path, PathBuf};

use rand::seq::IteratorRandom;
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Type(String);

#[derive(Deserialize, Debug, PartialEq)]
pub struct Color(String);

#[derive(Deserialize, Debug, PartialEq)]
pub struct Weight(f64);

#[derive(Deserialize, Debug, PartialEq)]
pub struct Height(f64);

#[derive(Deserialize, Debug, PartialEq)]
pub struct Generation(u8);

#[derive(Deserialize, Debug, PartialEq)]
pub struct Pokemon {
	pub name: String,
	pub height: Height,
	pub weight: Weight,
	pub types: Vec<Type>,
	pub color: Color,
	pub generation: Generation,
}

#[derive(Error, Debug)]
pub enum ReadingError {
	#[error("Error while trying to open the file")]
	SerdeError(#[from] serde_json::Error),
	#[error("Error while deserializing the pokemon")]
	OpeningError(#[from] std::io::Error),
	#[error("Error while choosing a random file")]
	RandomError,
}

pub fn get_random_pokemon(mut data_dir: PathBuf, lang: &str) -> Result<Pokemon, ReadingError>
{
	data_dir.push("generated_data");
	data_dir.push(lang);
	data_dir.push("pokedle");
	read_pokemon(get_random_file(&data_dir)?)
}

/*
	Private stuff
*/

fn read_pokemon<P>(filename: P) -> Result<Pokemon, ReadingError>
where
	P: AsRef<Path>,
{
	let file = File::open(filename)?;
	let pokemon: Pokemon = serde_json::from_reader(file)?;
	Ok(pokemon)
}

fn get_random_file<P>(dir_name: P) -> Result<PathBuf, ReadingError>
where
	P: AsRef<Path>,
{
	let mut rng = rand::thread_rng();
	let files = std::fs::read_dir(dir_name)?;
	let file : DirEntry = match files.choose(&mut rng) {
		Some(Ok(file)) => file,
		_ => return Err(ReadingError::RandomError),
	};
	let path = file.path().to_path_buf();
	Ok(path)
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn read_a_pokemon() {
		let chrysacier = Pokemon {
			name: String::from("Chrysacier"),
			height: Height(0.7),
			weight: Weight(9.9),
			types: vec![Type(String::from("Insecte"))],
			color: Color(String::from("Vert")),
			generation: Generation(1),
		};

		let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		d.push("poke_data/generated_data/fr/pokedle/11.json");
		let pokemon = read_pokemon(d).unwrap();
		assert_eq!(pokemon, chrysacier);
	}

	#[test]
	fn get_a_random_file() {
		let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		d.push("poke_data/generated_data/fr/pokedle");
		let random_file = get_random_file(d).unwrap();
		assert_eq!(random_file.extension().unwrap(), "json");
	}

	#[test]
	fn get_a_random_pokemon() {
		let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		d.push("poke_data");
		get_random_pokemon(d, "fr").unwrap();
	}
}
