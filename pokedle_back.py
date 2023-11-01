from fastapi import FastAPI
from fastapi.staticfiles import StaticFiles
from fastapi.middleware.cors import CORSMiddleware

import pokedle

p = pokedle.PokedleWrapper("poke_data")

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
	return p.guess(lang, pokemon_name).comparison

@app.get("/previous_pokemon")
async def get_previous_pokemon_to_guess_name(lang: str):
	return p.get_previous_pokemon_to_guess_name(lang)


app.mount("/", StaticFiles(directory="public", html=True), name="Something")

