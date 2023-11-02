from fastapi import FastAPI
from fastapi.staticfiles import StaticFiles
from fastapi.middleware.cors import CORSMiddleware

import pokedle

p = pokedle.Pokedle("poke_data")

app = FastAPI()
app.add_middleware(
	CORSMiddleware,
	allow_origins=["*"],
	allow_credentials=True,
	allow_methods=["*"],
	allow_headers=["*"],
)

@app.get("/names")
async def get_names(lang: str):
	return p.get_names(lang)

@app.post("/guess")
async def guess(lang: str, pokemon_name: str):
	comparison = p.guess(lang, pokemon_name)
	pokemon = p.get_pokemon_by_name(lang, pokemon_name)
	if len(pokemon.types) == 1:
		pokemon_types_string = pokemon.types[0].to_string()
	else:
		pokemon_types_string = f"{pokemon.types[0].to_string()}, {pokemon.types[1].to_string()}"

	return {
		"success": comparison.success,
		"height": {
			"pokemon": pokemon.height.to_string(),
			"comparison": comparison.height.to_string()
		},
		"weight": {
			"pokemon": pokemon.weight.to_string(),
			"comparison": comparison.weight.to_string()
		},
		"types": {
			"pokemon": pokemon_types_string,
			"comparison": comparison.types.to_string()
		},
		"color": {
			"pokemon": pokemon.color.to_string(),
			"comparison": comparison.color.to_string()
		},
		"generation": {
			"pokemon": pokemon.generation.to_string(),
			"comparison": comparison.generation.to_string()
		},
	}

@app.get("/previous_pokemon")
async def get_previous_pokemon_to_guess_name(lang: str):
	return p.get_previous_pokemon_to_guess_name(lang)

app.mount("/", StaticFiles(directory="public", html=True), name="Something")

