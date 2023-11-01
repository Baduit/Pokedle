use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use pokedle_core::{GuessResult, Pokedle};

#[pymodule]
#[pyo3(name = "pokedle")]
fn pokedle_module(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PokedleWrapper>()?;
    m.add_class::<PythonGuessResult>()?;
    Ok(())
}

#[pyclass]
pub struct PythonGuessResult {
    #[pyo3(get)]
    pub comparison: Option<[String; 5]>,
}


impl PythonGuessResult {
    pub fn new(guess_result: GuessResult) -> Self {
        match guess_result {
            GuessResult::Success => PythonGuessResult {
                comparison: None,
            },
            GuessResult::Failure(pokemon_comparison) => PythonGuessResult {
                comparison: Some(pokemon_comparison.to_array_of_string()),
            },
        }
    }

}

#[pymethods]
impl PythonGuessResult {
    pub fn is_success(&self) -> bool {
        self.comparison.is_some()
    }
}

#[pyclass]
struct PokedleWrapper {
    game_logic: Pokedle,
}

#[pymethods]
impl PokedleWrapper {
    #[new]
    pub fn py_new(data_dir: &str) -> PyResult<Self> {
        match Pokedle::new(data_dir) {
            Ok(game_logic) => Ok(PokedleWrapper { game_logic }),
            Err(error) => Err(PyValueError::new_err(format!("{}", error))),
        }
    }

    pub fn guess(&mut self, lang: &str, pokemon_name: &str) -> PyResult<PythonGuessResult> {
        match self.game_logic.guess(lang, pokemon_name) {
            Ok(guess_result) => Ok(PythonGuessResult::new(guess_result)),
            Err(error) => Err(PyValueError::new_err(format!("{}", error))),
        }
    }

    pub fn get_names(&self, lang: &str) -> PyResult<Vec<String>> {
        match self.game_logic.get_names(lang) {
            Ok(names) => Ok(names.clone()),
            Err(error) => Err(PyValueError::new_err(format!("{}", error))),
        }
    }

    pub fn get_previous_pokemon_to_guess_name(&self, lang: &str) -> PyResult<Option<String>> {
        match self.game_logic.get_previous_pokemon_to_guess_name(lang) {
            Ok(name) => Ok(name),
            Err(error) => Err(PyValueError::new_err(format!("{}", error))),
        }
    }
}
