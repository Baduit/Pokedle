use pokemon::get_all_pokemons;
use rand::distributions::Uniform;
use rand::Rng;
use std::collections::BTreeMap;
use std::iter::zip;
use std::path::Path;

use chrono::prelude::*;
use thiserror::Error;

mod pokemon;
pub use pokemon::{
    compare_pokemons, get_names, Color, ColorComparison, Generation, Height, Lang,
    NumberComparison, Pokemon, PokemonComparison, ReadingError, Type, TypesComparison, Weight,
};

#[derive(Error, Debug)]
pub enum PokedleError {
    #[error("The language {0} does not exist")]
    LangDoesNotExist(String),
    #[error("The pokemon {0} does not exist")]
    PokemonDoesNotExist(String),
}

struct PokemonHandler {
    pokemon_names: Vec<String>,
    pokemons: Vec<Pokemon>,
    daily_pokemon_index: usize,
    last_pokemon_update: DateTime<Utc>,
    previous_daily_pokemon_index: Option<usize>,
    // Todo: add an id to know if the pokemon changed while the player was playing
}

impl PokemonHandler {
    pub fn new(pokemon_names: Vec<String>, pokemons: Vec<Pokemon>) -> PokemonHandler {
        let number_of_pokemons = pokemons.len();

        // We want to set the generation time in the night
        let current_datetime = Utc::now();
        let first_generation = Utc
            .with_ymd_and_hms(
                current_datetime.year(),
                current_datetime.month(),
                current_datetime.day(),
                6,
                0,
                0,
            )
            .unwrap();

        PokemonHandler {
            pokemon_names,
            pokemons,
            daily_pokemon_index: PokemonHandler::get_random_pokemon_index(number_of_pokemons),
            last_pokemon_update: first_generation,
            previous_daily_pokemon_index: None
        }
    }

    fn get_random_pokemon_index(number_of_pokemons: usize) -> usize {
        let mut rng = rand::thread_rng();
        let pokemon_distribution = Uniform::new(0, number_of_pokemons);
        rng.sample(pokemon_distribution)
    }

    pub fn get_pokemon_by_name(&self, name: &str) -> Result<&Pokemon, PokedleError> {
        match self.pokemons.iter().filter(|p| p.name == name).next() {
            Some(pokemon) => Ok(pokemon),
            None => Err(PokedleError::PokemonDoesNotExist(String::from(name))),
        }
    }

    pub fn get_daily_pokemon(&self) -> &Pokemon {
        // Safe to unwrap, because the index is generated from the size of the vec and the vec has a constant size
        self.pokemons.get(self.daily_pokemon_index).unwrap()
    }

    pub fn update_daily_pokemon_if_needed(&mut self) {
        if self.is_update_needed() {
            self.previous_daily_pokemon_index = Some(self.daily_pokemon_index);
            self.daily_pokemon_index =
                PokemonHandler::get_random_pokemon_index(self.pokemons.len());
            self.last_pokemon_update = Utc::now();
        }
    }

    fn is_update_needed(&self) -> bool {
        let diff_time = Utc::now() - self.last_pokemon_update;
        diff_time.num_days() >= 1
    }
}

pub struct Pokedle {
    handlers: BTreeMap<Lang, PokemonHandler>,
}

#[derive(Error, Debug)]
pub enum PokedleInitError {
    #[error("Can't load the data to initialize Pokedle")]
    DataLoadingError(#[from] ReadingError),
    #[error("The loaded data are incoherent")]
    IncoherentData,
}

#[derive(Debug, PartialEq)]
pub enum GuessResult {
    Success,
    Failure(PokemonComparison),
}

impl Pokedle {
    pub fn new<P>(pokle_dir: P) -> Result<Pokedle, PokedleInitError>
    where
        P: AsRef<Path>,
    {
        let names = get_names(pokle_dir.as_ref().to_path_buf())?;
        let pokemons = get_all_pokemons(pokle_dir.as_ref().to_path_buf())?;

        let mut pokedle = Pokedle {
            handlers: BTreeMap::new(),
        };

        for ((name_lang, names), (pokemon_lang, pokemons)) in zip(names, pokemons) {
            if name_lang != pokemon_lang {
                return Err(PokedleInitError::IncoherentData);
            }

            pokedle
                .handlers
                .insert(name_lang, PokemonHandler::new(names, pokemons));
        }
        Ok(pokedle)
    }

    pub fn guess(&mut self, lang: &str, pokemon_name: &str) -> Result<GuessResult, PokedleError> {
        let handler = match self.handlers.get_mut(lang) {
            Some(handler) => handler,
            None => return Err(PokedleError::LangDoesNotExist(String::from(lang))),
        };
        handler.update_daily_pokemon_if_needed();

        let daily_pokemon = handler.get_daily_pokemon();
        if pokemon_name == daily_pokemon.name {
            Ok(GuessResult::Success)
        } else {
            let input_pokemon = handler.get_pokemon_by_name(pokemon_name)?;
            let comparison = compare_pokemons(input_pokemon, daily_pokemon);
            Ok(GuessResult::Failure(comparison))
        }
    }

    pub fn get_names(&self, lang: &str) -> Result<&Vec<String>, PokedleError> {
        match self.handlers.get(lang) {
            Some(handler) => Ok(&handler.pokemon_names),
            None => Err(PokedleError::LangDoesNotExist(String::from(lang))),
        }
    }

    pub fn get_previous_pokemon_to_guess_name(&self, lang: &str) -> Result<Option<String>, PokedleError> {
        let handler = match self.handlers.get(lang) {
            Some(handler) => handler,
            None => return Err(PokedleError::LangDoesNotExist(String::from(lang))),
        };

        match handler.previous_daily_pokemon_index {
            // Ok to unwrap because the index is generated within the bould of this vector
            Some(index) => Ok(Some(handler.pokemon_names.get(index).unwrap().clone())),
            None => Ok(None),
        }
    }
}

/*
    Tests
*/
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /*
     ** Pokedle tests
     */
    #[test]
    fn pokedle_creatation_simplified_data() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("../small_test_data");
        Pokedle::new(d).unwrap();
    }

    #[test]
    fn pokedle_creatation_real_data() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("../poke_data");
        Pokedle::new(d).unwrap();
    }

    #[test]
    fn game_scenario() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("../small_test_data");
        let mut pokedle = Pokedle::new(d).unwrap();
        // Cheat a bit to know which pokemon we are trying to guess
        pokedle
            .handlers
            .get_mut("fr")
            .expect("Error in the test, not in the code")
            .daily_pokemon_index = 0;

        pokedle
            .get_names("lol")
            .expect_err("'lol' should not exist");
        pokedle.get_names("fr").unwrap();

        pokedle
            .guess("lo", "Chenipan")
            .expect_err("'lo' should not be a valid language");
        pokedle
            .guess("fr", "Sacha")
            .expect_err("'Sacha' should not be a pokemon");
        assert_eq!(
            pokedle.guess("fr", "Herbizarre").unwrap(),
            GuessResult::Failure(PokemonComparison {
                height: NumberComparison::Higher,
                weight: NumberComparison::Higher,
                types: TypesComparison::Equal,
                color: ColorComparison::Equal,
                generation: NumberComparison::Equal,
            })
        );
        assert_eq!(
            pokedle.guess("fr", "Bulbizarre").unwrap(),
            GuessResult::Success
        );
    }

    /*
     ** PokemonHandler tests
     */
    #[test]
    fn pokemon_handler_creation() {
        let (names, pokemons) = generate_dummy_pokemon_data();
        let handler = PokemonHandler::new(names, pokemons.clone());
        let daily_pokemon = handler.get_daily_pokemon();
        assert!(pokemons.contains(daily_pokemon));
    }

    #[test]
    fn pokemon_handler_get_pokemon_by_name() {
        let (names, pokemons) = generate_dummy_pokemon_data();
        let handler = PokemonHandler::new(names, pokemons.clone());

        assert_eq!(
            handler
                .get_pokemon_by_name("Chrysacier")
                .expect("Oh no, the pokemon is not found"),
            pokemons
                .get(0)
                .expect("The error is in the test, not in the code")
        );

        assert_eq!(
            handler
                .get_pokemon_by_name("ChrysacierBis")
                .expect("Oh no, the pokemon is not found"),
            pokemons
                .get(1)
                .expect("The error is in the test, not in the code")
        );

        assert_eq!(
            handler
                .get_pokemon_by_name("BlagueSurLesDaron-ne-s")
                .expect("Oh no, the pokemon is not found"),
            pokemons
                .get(2)
                .expect("The error is in the test, not in the code")
        );

        assert_eq!(
            handler
                .get_pokemon_by_name("Blanche")
                .expect("Oh no, the pokemon is not found"),
            pokemons
                .get(3)
                .expect("The error is in the test, not in the code")
        );

        assert_eq!(
            handler
                .get_pokemon_by_name("Noirette")
                .expect("Oh no, the pokemon is not found"),
            pokemons
                .get(4)
                .expect("The error is in the test, not in the code")
        );
    }

    #[test]
    fn pokemon_handler_update() {
        let (names, pokemons) = generate_dummy_pokemon_data();
        let mut handler = PokemonHandler::new(names, pokemons.clone());
        let first_index = handler.daily_pokemon_index;

        // Do it a lot, to be sure that's not just luck, theorically it is still possible but it would really improbable
        for _ in [0..100] {
            handler.update_daily_pokemon_if_needed();
            // The creation just happened, so it should not change
            assert_eq!(first_index, handler.daily_pokemon_index);
        }

        // Change the last update so it is at least one day in the past, now it should change
        let current_datetime = Utc::now();
        handler.last_pokemon_update = Utc
            .with_ymd_and_hms(
                current_datetime.year(),
                current_datetime.month(),
                current_datetime.day() - 1,
                6,
                0,
                0,
            )
            .unwrap();
        // Do it until it is different (because there is random) with a limitation to not have an infinite loop if it fails
        let mut index_changed = false;
        for _ in [0..100] {
            handler.update_daily_pokemon_if_needed();
            // The creation just happened, so it should not change
            if first_index != handler.daily_pokemon_index {
                assert_eq!(first_index, handler.previous_daily_pokemon_index.unwrap());
                index_changed = true;
                break;
            }
        }
        assert!(index_changed);
    }

    fn generate_dummy_pokemon_data() -> (Vec<String>, Vec<Pokemon>) {
        let pokemons = vec![
            Pokemon {
                name: String::from("Chrysacier"),
                height: Height(0.7),
                weight: Weight(9.9),
                types: vec![Type(String::from("Insecte"))],
                color: Color(String::from("Vert")),
                generation: Generation(1),
            },
            Pokemon {
                name: String::from("ChrysacierBis"),
                height: Height(0.7),
                weight: Weight(9.9),
                types: vec![Type(String::from("Insecte"))],
                color: Color(String::from("Vert")),
                generation: Generation(12),
            },
            Pokemon {
                name: String::from("BlagueSurLesDaron-ne-s"),
                height: Height(0.7),
                weight: Weight(9.9),
                types: vec![Type(String::from("Insecte"))],
                color: Color(String::from("Vert")),
                generation: Generation(5),
            },
            Pokemon {
                name: String::from("Blanche"),
                height: Height(0.3),
                weight: Weight(3.2),
                types: vec![Type(String::from("Normal"))],
                color: Color(String::from("Blanc")),
                generation: Generation(2),
            },
            Pokemon {
                name: String::from("Noirette"),
                height: Height(0.3),
                weight: Weight(4.1),
                types: vec![Type(String::from("Normal"))],
                color: Color(String::from("Noir")),
                generation: Generation(2),
            },
        ];

        let names: Vec<String> = pokemons.iter().map(|p| p.name.clone()).collect();
        (names, pokemons)
    }
}
