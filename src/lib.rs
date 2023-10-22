use rand::distributions::Uniform;
use rand::Rng;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::prelude::*;
use thiserror::Error;

mod pokemon;
pub use pokemon::{get_names, Lang, Pokemon, ReadingError};

// En fait j'ai âpas d'id ou d'état coté serveur, on peut faire comme loldle et essai infini
// Du coup le seul truc à stocker sera le pokémon du jour

struct PokemonHandler {
    pokemon_names: Vec<String>,
    pokemons: Vec<Pokemon>,
    daily_pokemon_index: usize,
    last_pokemon_update: DateTime<Utc>,
}

// Todo make proper error types
type UnknownPokemon = ();

/* #[derive(Error, Debug)]
enum PokemonHandlerCreationError {
    #[error("Error while to read data files")]
    ReadingError(#[from] ReadingError),
    #[error("Error while trying to find a random pokemon")]
    RandomFailure,
} */

impl PokemonHandler {
    pub fn new(pokemon_names: Vec<String>, pokemons: Vec<Pokemon>) -> PokemonHandler {
        let number_of_pokemons = pokemons.len();

        // We want to set the generation time in the night
        let mut current_datetime = Utc::now();
        let first_generation = Utc.with_ymd_and_hms(
            current_datetime.year(),
            current_datetime.month(),
            current_datetime.day(),
            6,
            0,
            0,
        ).unwrap();

        PokemonHandler {
            pokemon_names,
            pokemons,
            daily_pokemon_index: PokemonHandler::get_random_pokemon_index(number_of_pokemons),
            last_pokemon_update: first_generation,
        }
    }

    fn get_random_pokemon_index(number_of_pokemons: usize) -> usize {
        let mut rng = rand::thread_rng();
        let pokemon_distribution = Uniform::new(0, number_of_pokemons);
        rng.sample(pokemon_distribution)
    }

    pub fn get_pokemon_by_name(&self, name: &str) -> Result<&Pokemon, UnknownPokemon> {
        match self.pokemons.iter().filter(|p| p.name == name).next() {
            Some(pokemon) => Ok(pokemon),
            None => Err(()),
        }
    }

    pub fn get_daily_pokemon(&self) -> &Pokemon {
        // Safe to unwrap, because the index is generated from the size of the vec and the vec has a constant size
        self.pokemons.get(self.daily_pokemon_index).unwrap()
    }

    pub fn update_daily_pokemon_if_needed(&mut self) {
		if self.is_update_needed() {
			self.daily_pokemon_index = PokemonHandler::get_random_pokemon_index(self.pokemons.len());
			self.last_pokemon_update = Utc::now();
		}
	}

    fn is_update_needed(&self) -> bool {
		let diff_time = Utc::now() - self.last_pokemon_update;
        diff_time.num_days() >= 1 
    }
}

struct Pokedle {
    handlers: HashMap<Lang, PokemonHandler>,
}

impl Pokedle {
    pub fn guess() {}

    pub fn get_names() {}
}

fn compare_pokemons() {}
