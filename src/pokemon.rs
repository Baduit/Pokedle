use fs::File;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

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

#[pyclass]
#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Pokemon {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub height: Height,
    #[pyo3(get)]
    pub weight: Weight,
    #[pyo3(get)]
    pub types: Vec<Type>,
    #[pyo3(get)]
    pub color: Color,
    #[pyo3(get)]
    pub generation: Generation,
}

#[derive(Deserialize, Debug, PartialEq)]
struct Metadata {
    names: Vec<String>,
}

#[pyclass]
#[derive(Debug, PartialEq, Clone)]
pub enum NumberComparison {
    Higher,
    Lower,
    Equal,
}

#[pymethods]
impl NumberComparison {
    fn to_string(&self) -> String {
        match self {
            Self::Higher => String::from("higher"),
            Self::Lower => String::from("lower"),
            Self::Equal => String::from("equal"),
        }
    }
}

#[pyclass]
#[derive(Debug, PartialEq, Clone)]
pub enum TypesComparison {
    Different,
    Equal,
    PartiallyEqual,
}

#[pymethods]
impl TypesComparison {
    fn to_string(&self) -> String {
        match self {
            Self::Different => String::from("different"),
            Self::PartiallyEqual => String::from("partially_equal"),
            Self::Equal => String::from("equal"),
        }
    }
}

#[pyclass]
#[derive(Debug, PartialEq, Clone)]
pub enum ColorComparison {
    Different,
    Equal,
}

#[pymethods]
impl ColorComparison {
    fn to_string(&self) -> String {
        match self {
            Self::Different => String::from("different"),
            Self::Equal => String::from("equal"),
        }
    }
}

#[pyclass]
#[derive(Debug, PartialEq, Clone)]
pub struct PokemonComparison {
    #[pyo3(get)]
    pub success: bool,
    #[pyo3(get)]
    pub height: NumberComparison,
    #[pyo3(get)]
    pub weight: NumberComparison,
    #[pyo3(get)]
    pub types: TypesComparison,
    #[pyo3(get)]
    pub color: ColorComparison,
    #[pyo3(get)]
    pub generation: NumberComparison,
}



pub fn compare_pokemons(guess: &Pokemon, pokemon_to_guess: &Pokemon) -> PokemonComparison {
    if guess.name == pokemon_to_guess.name {
        return PokemonComparison {
            success: true,
            height: NumberComparison::Equal,
            weight: NumberComparison::Equal,
            types: TypesComparison::Equal,
            color: ColorComparison::Equal,
            generation: NumberComparison::Equal,
        }
    }

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
        success: false,
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

#[pyclass]
#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Type(pub String);

#[pymethods]
impl Type {
    pub fn to_string(&self) -> String {
        format!("{}", self.0)
    }
}

#[pyclass]
#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Color(pub String);

#[pymethods]
impl Color {
    pub fn to_string(&self) -> String {
        format!("{}", self.0)
    }
}

#[pyclass]
#[derive(Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct Weight(pub f64);

#[pymethods]
impl Weight {
    pub fn to_string(&self) -> String {
        format!("{}", self.0)
    }
}

#[pyclass]
#[derive(Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct Height(pub f64);

#[pymethods]
impl Height {
    pub fn to_string(&self) -> String {
        format!("{}", self.0)
    }
}

#[pyclass]
#[derive(Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct Generation(pub u8);

#[pymethods]
impl Generation {
    pub fn to_string(&self) -> String {
        format!("{}", self.0)
    }
}

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
            success: true,
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
            success: false,
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
            success: false,
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
            success: false,
            height: NumberComparison::Higher,
            weight: NumberComparison::Higher,
            types: TypesComparison::Different,
            color: ColorComparison::Different,
            generation: NumberComparison::Higher,
        };
        assert_eq!(compare_pokemons(&chrysacier, &my_creature), expected_result);
    }
}
