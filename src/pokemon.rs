use fs::File;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[derive(Error, Debug)]
pub enum ReadingError {
    #[error("Error while trying to open the file")]
    SerdeError(#[from] serde_json::Error),
    #[error("Error while deserializing")]
    OpeningError(#[from] std::io::Error),
    #[error("Invalid pokedata structure")]
    WrongFileStructure,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
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

impl fmt::Display for NumberComparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Higher => write!(f, "higher"),
            Self::Lower => write!(f, "lower"),
            Self::Equal => write!(f, "equal"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TypesComparison {
    Different,
    Equal,
    PartiallyEqual,
}

impl fmt::Display for TypesComparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Different => write!(f, "different"),
            Self::PartiallyEqual => write!(f, "partially_equal"),
            Self::Equal => write!(f, "equal"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ColorComparison {
    Different,
    Equal,
}

impl fmt::Display for ColorComparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Different => write!(f, "different"),
            Self::Equal => write!(f, "equal"),
        }
    }
}

#[pyclass]
#[derive(Debug, PartialEq)]
pub struct PokemonComparison {
    pub height: NumberComparison,
    pub weight: NumberComparison,
    pub types: TypesComparison,
    pub color: ColorComparison,
    pub generation: NumberComparison,
}

impl PokemonComparison {
    pub fn to_array_of_string(&self) -> [String; 5] {
        [
            format!("{}", self.height),
            format!("{}", self.weight),
            format!("{}", self.types),
            format!("{}", self.color),
            format!("{}", self.generation),
        ]
    }
}

// It does not check the name because there is no need to compare pokemons if the name is the same
pub fn compare_pokemons(guess: &Pokemon, pokemon_to_guess: &Pokemon) -> PokemonComparison {
    let height = if guess.height == pokemon_to_guess.height {
        NumberComparison::Equal
    } else if guess.height < pokemon_to_guess.height {
        NumberComparison::Higher
    } else {
        NumberComparison::Lower
    };

    let weight = if guess.weight == pokemon_to_guess.weight {
        NumberComparison::Equal
    } else if guess.weight < pokemon_to_guess.weight {
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
    } else if guess.generation < pokemon_to_guess.generation {
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

pub fn get_names(mut data_dir: PathBuf) -> Result<BTreeMap<Lang, Vec<String>>, ReadingError> {
    let mut names = BTreeMap::new();
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
) -> Result<BTreeMap<Lang, Vec<Pokemon>>, ReadingError> {
    let mut pokemons_by_lang: BTreeMap<Lang, Vec<Pokemon>> = BTreeMap::new();

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

/*
    Small types
*/
pub type Lang = String;

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Type(pub String);

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Color(pub String);

#[derive(Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct Weight(pub f64);

#[derive(Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct Height(pub f64);

#[derive(Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct Generation(pub u8);

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

        let good_guess: PokemonComparison = PokemonComparison {
            height: NumberComparison::Equal,
            weight: NumberComparison::Equal,
            types: TypesComparison::Equal,
            color: ColorComparison::Equal,
            generation: NumberComparison::Equal,
        };

        assert_eq!(compare_pokemons(&chrysacier, &chrysacier_bis), good_guess);
    }

    #[test]
    fn test_compare_partial_equal_type_lower_numbers() {
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
            height: NumberComparison::Higher,
            weight: NumberComparison::Higher,
            types: TypesComparison::PartiallyEqual,
            color: ColorComparison::Different,
            generation: NumberComparison::Higher,
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
            height: NumberComparison::Higher,
            weight: NumberComparison::Higher,
            types: TypesComparison::Different,
            color: ColorComparison::Different,
            generation: NumberComparison::Higher,
        };
        assert_eq!(compare_pokemons(&chrysacier, &my_creature), expected_result);
    }
}
