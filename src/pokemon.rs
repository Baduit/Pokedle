use fs::File;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use rand::seq::IteratorRandom;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReadingError {
    #[error("Error while trying to open the file")]
    SerdeError(#[from] serde_json::Error),
    #[error("Error while deserializing")]
    OpeningError(#[from] std::io::Error),
    #[error("Error while choosing a random file")]
    RandomError,
    #[error("Invalid pokedata structure")]
    WrongFileStructure,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Pokemon {
    pub name: String,
    pub height: Height,
    pub weight: Weight,
    pub types: Vec<Type>,
    pub color: Color,
    pub generation: Generation,
}

#[derive(Deserialize, Debug, PartialEq)]
struct Metadata {
    names: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum NumberComparison {
    Higher,
    Lower,
    Equal,
}

#[derive(Debug, PartialEq)]
pub enum TypesComparison {
    Different,
    Equal,
    PartiallyEqual,
}

#[derive(Debug, PartialEq)]
pub enum ColorComparison {
    Different,
    Equal,
}

#[derive(Debug, PartialEq)]
pub struct PokemonComparison {
    pub height: NumberComparison,
    pub weight: NumberComparison,
    pub types: TypesComparison,
    pub color: ColorComparison,
    pub generation: NumberComparison,
}

pub const COMPARISON_ON_GOOD_GUESS: PokemonComparison = PokemonComparison {
    height: NumberComparison::Equal,
    weight: NumberComparison::Equal,
    types: TypesComparison::Equal,
    color: ColorComparison::Equal,
    generation: NumberComparison::Equal,
};

// It does not check the name because there is no need to compare pokemons if the name is the same
// And in this case this function should not be called and the constant COMPARISON_ON_GOOD_GUESS should be used instead
pub fn compare_pokemons(guess: &Pokemon, pokemon_to_guess: &Pokemon) -> PokemonComparison {
    let height = if guess.height == pokemon_to_guess.height {
        NumberComparison::Equal
    } else if guess.height > pokemon_to_guess.height {
        NumberComparison::Higher
    } else {
        NumberComparison::Lower
    };

    let weight = if guess.weight == pokemon_to_guess.weight {
        NumberComparison::Equal
    } else if guess.weight > pokemon_to_guess.weight {
        NumberComparison::Higher
    } else {
        NumberComparison::Lower
    };

    let types = if guess.types == pokemon_to_guess.types {
        TypesComparison::Equal
    } else {
        let mut common_type_found = false;
        for t in guess.types.iter() {
            if pokemon_to_guess.types.contains(t) {
                common_type_found = true;
            }
        }

        if common_type_found {
            TypesComparison::PartiallyEqual
        } else {
            TypesComparison::Different
        }
    };

    let color = if guess.color == pokemon_to_guess.color {
        ColorComparison::Equal
    } else {
        ColorComparison::Different
    };

    let generation = if guess.generation == pokemon_to_guess.generation {
        NumberComparison::Equal
    } else if guess.generation > pokemon_to_guess.generation {
        NumberComparison::Higher
    } else {
        NumberComparison::Lower
    };

    PokemonComparison {
        height,
        weight,
        types,
        color,
        generation,
    }
}

pub fn get_names(mut data_dir: PathBuf) -> Result<HashMap<Lang, Vec<String>>, ReadingError> {
    let mut names = HashMap::new();
    data_dir.push("generated_data");
    for dir in std::fs::read_dir(data_dir)? {
        let dir = dir?;
        let lang = get_lang(dir.path().file_name())?;
        let metadata = get_metadata(dir.path().to_path_buf())?;
        names.insert(lang, metadata.names);
    }
    Ok(names)
}

pub fn get_all_pokemons(
    mut data_dir: PathBuf,
) -> Result<HashMap<Lang, Vec<Pokemon>>, ReadingError> {
    let mut pokemons_by_lang: HashMap<Lang, Vec<Pokemon>> = HashMap::new();

    data_dir.push("generated_data");

    for lang_dir in std::fs::read_dir(data_dir)? {
        let lang_dir = lang_dir?;
        let lang = get_lang(lang_dir.path().file_name())?;

        let mut lang_dir = lang_dir.path().to_path_buf();
        lang_dir.push("pokedle");

        let mut pokemons = Vec::new();
        for poke_file in std::fs::read_dir(lang_dir)? {
            let poke_file = poke_file?;
            pokemons.push(read_pokemon(poke_file.path())?);
        }

        pokemons_by_lang.insert(lang, pokemons);
    }

    Ok(pokemons_by_lang)
}

pub fn get_random_pokemon(mut data_dir: PathBuf, lang: &str) -> Result<Pokemon, ReadingError> {
    data_dir.push("generated_data");
    data_dir.push(lang);
    data_dir.push("pokedle");
    read_pokemon(get_random_file(&data_dir)?)
}

/*
    Small types
*/
pub type Lang = String;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Type(String);

#[derive(Deserialize, Debug, PartialEq)]
pub struct Color(String);

#[derive(Deserialize, Debug, PartialEq, PartialOrd)]
pub struct Weight(f64);

#[derive(Deserialize, Debug, PartialEq, PartialOrd)]
pub struct Height(f64);

#[derive(Deserialize, Debug, PartialEq, PartialOrd)]
pub struct Generation(u8);

/*
    Private stuff
*/

fn get_lang(filename: Option<&OsStr>) -> Result<Lang, ReadingError> {
    let lang = match filename {
        Some(lang) => lang.to_str(),
        _ => return Err(ReadingError::WrongFileStructure),
    };
    let lang = match lang {
        Some(lang) => lang,
        _ => return Err(ReadingError::WrongFileStructure),
    };
    Ok(lang.to_string())
}

fn get_metadata(mut lang_dir: PathBuf) -> Result<Metadata, ReadingError> {
    lang_dir.push("metadata.json");
    let file = File::open(lang_dir)?;
    let metadata: Metadata = serde_json::from_reader(file)?;
    Ok(metadata)
}

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
    let file: DirEntry = match files.choose(&mut rng) {
        Some(Ok(file)) => file,
        _ => return Err(ReadingError::RandomError),
    };
    let path = file.path().to_path_buf();
    Ok(path)
}

/*
    Tests
*/
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

    #[test]
    fn get_all_names() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("poke_data");
        let names = get_names(d).unwrap();
        assert_eq!(names["fr"][0], "Bulbizarre");
        assert_eq!(names["de"][0], "Bisasam");
    }

    #[test]
    fn get_all_pokemons_of_all_lang() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("poke_data");
        let names = get_all_pokemons(d).unwrap();
        assert_eq!(names["fr"][0].name, "Bulbizarre");
        assert_eq!(names["de"][0].name, "Bisasam");
    }

    #[test]
    fn test_compare_same() {
        // This is not a real use case, but it allows to test equality on all fields
        let chrysacier = Pokemon {
            name: String::from("Chrysacier"),
            height: Height(0.7),
            weight: Weight(9.9),
            types: vec![Type(String::from("Insecte"))],
            color: Color(String::from("Vert")),
            generation: Generation(1),
        };

        let chrysacier_bis = Pokemon {
            name: String::from("Chrysacier"),
            height: Height(0.7),
            weight: Weight(9.9),
            types: vec![Type(String::from("Insecte"))],
            color: Color(String::from("Vert")),
            generation: Generation(1),
        };

        assert_eq!(compare_pokemons(&chrysacier, &chrysacier_bis), COMPARISON_ON_GOOD_GUESS);
    }

    #[test]
    fn test_compare_partial_equal_type_higher_numbers() {
        // This is not a real use case, but it allows to test equality on all fields
        let chrysacier = Pokemon {
            name: String::from("Chrysacier"),
            height: Height(0.7),
            weight: Weight(9.9),
            types: vec![Type(String::from("Insecte"))],
            color: Color(String::from("Vert")),
            generation: Generation(1),
        };

        let my_creature = Pokemon {
            name: String::from("my_creature"),
            height: Height(0.8),
            weight: Weight(10.0),
            types: vec![Type(String::from("Insecte")), Type(String::from("Feu"))],
            color: Color(String::from("Rouge")),
            generation: Generation(2),
        };

        let expected_result = PokemonComparison {
            height: NumberComparison::Higher,
            weight: NumberComparison::Higher,
            types: TypesComparison::PartiallyEqual,
            color: ColorComparison::Different,
            generation: NumberComparison::Higher,
        };

        assert_eq!(compare_pokemons(&my_creature, &chrysacier), expected_result);
    }

    #[test]
    fn test_compare_partial_equal_reverse() {
        // This is not a real use case, but it allows to test equality on all fields
        let chrysacier = Pokemon {
            name: String::from("Chrysacier"),
            height: Height(0.7),
            weight: Weight(9.9),
            types: vec![Type(String::from("Insecte"))],
            color: Color(String::from("Vert")),
            generation: Generation(1),
        };

        let my_creature = Pokemon {
            name: String::from("my_creature"),
            height: Height(0.8),
            weight: Weight(10.0),
            types: vec![Type(String::from("Insecte")), Type(String::from("Feu"))],
            color: Color(String::from("Rouge")),
            generation: Generation(2),
        };

        let expected_result = PokemonComparison {
            height: NumberComparison::Lower,
            weight: NumberComparison::Lower,
            types: TypesComparison::PartiallyEqual,
            color: ColorComparison::Different,
            generation: NumberComparison::Lower,
        };

        assert_eq!(compare_pokemons(&chrysacier, &my_creature), expected_result);
    }

    #[test]
    fn test_compare_totally_different() {
        // This is not a real use case, but it allows to test equality on all fields
        let chrysacier = Pokemon {
            name: String::from("Chrysacier"),
            height: Height(0.7),
            weight: Weight(9.9),
            types: vec![Type(String::from("Insecte"))],
            color: Color(String::from("Vert")),
            generation: Generation(1),
        };

        let my_creature = Pokemon {
            name: String::from("my_creature"),
            height: Height(0.8),
            weight: Weight(10.0),
            types: vec![Type(String::from("Acier")), Type(String::from("Feu"))],
            color: Color(String::from("Rouge")),
            generation: Generation(2),
        };

        let expected_result = PokemonComparison {
            height: NumberComparison::Lower,
            weight: NumberComparison::Lower,
            types: TypesComparison::Different,
            color: ColorComparison::Different,
            generation: NumberComparison::Lower,
        };
        assert_eq!(compare_pokemons(&chrysacier, &my_creature), expected_result);
    }
}
