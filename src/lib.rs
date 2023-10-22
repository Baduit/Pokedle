use std::collections::HashMap;

mod pokemon;
pub use pokemon::{get_names, Lang, Pokemon, ReadingError};

// En fait j'ai âpas d'id ou d'état coté serveur, on peut faire comme loldle et essai infini
// Du coup le seul truc à stocker sera le pokémon du jour

struct PokemonHandler {
    cached_pokemons: HashMap<Lang, Vec<Pokemon>>,
	daily_pokemon: Pokemon,
	// time of the last update
}

impl PokemonHandler {
	pub fn get_random_pokemon() {

	}

	pub fn get_pokemon() {

	}

	pub fn update_daily_pokemon_if_needed() {

	}

	fn is_update_needed() -> bool {
		false
	}
}

struct Pokedle {
    names: HashMap<Lang, Vec<String>>,
    handlers: HashMap<Lang, PokemonHandler>,
}

impl Pokedle {
	pub fn guess() {

	}

	pub fn get_names() {

	}
}

fn compare_pokemons() {

}

