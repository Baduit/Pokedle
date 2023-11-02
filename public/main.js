let pokemon_names = [];

function onKeyPressed(event) {
	const enterKeyCode = 13;


	if (event.keyCode == enterKeyCode) {
		try_guess_pokemon();
	}
}

function is_local() {
	return window.location.href.startsWith("file") || window.location.href.indexOf("localhost") != -1;
}

function add_cell(row, comparison, cell_text) {
	let newCell = row.insertCell();

	if (comparison === "equal") {
		newCell.classList.add("equal_result");
	} else if (comparison === "partially_equal") {
		newCell.classList.add("partially_equal_result");
	} else {
		newCell.classList.add("different_result");
		if (comparison === "higher") {
			cell_text += " ↑"
		} else if (comparison === "lower") {
			cell_text += " ↓"
		}
	}

	let text_element = document.createTextNode(cell_text);
	newCell.appendChild(text_element);
}

function add_row_in_result_table(results) {
	console.log(results)

	let tbodyRef = document.getElementById('result_table').getElementsByTagName('tbody')[0];
	let newRow = tbodyRef.insertRow();
	add_cell(newRow, results.height.comparison, results.height.pokemon)
	add_cell(newRow, results.weight.comparison, results.weight.pokemon)
	add_cell(newRow, results.types.comparison, results.types.pokemon)
	add_cell(newRow, results.color.comparison, results.color.pokemon)
	add_cell(newRow, results.generation.comparison, results.generation.pokemon)

	if (results.success) {
		let a_surprise_for_later = document.getElementById("a_surprise_for_later");
		a_surprise_for_later.textContent = "You won !"
	}
}

async function try_guess_pokemon() {
	let pokemon_name = document.getElementById("title_input").value;
	// Capitalize first character
	pokemon_name = pokemon_name.charAt(0).toUpperCase() + pokemon_name.slice(1);
	if (pokemon_name == "") {
		return;
	} else if (!pokemon_names.includes(pokemon_name)) {
		alert('This pokemon does not exist.');
	}

	const options = {
		method: 'POST'
	};

	let url;
	if (is_local()) {
		url_guess = new URL("http://localhost:3412/guess");
	} else {
		url_guess = new URL("https://pokedle.baduit.eu/guess");
	}

	url_guess.searchParams.append("pokemon_name", pokemon_name);
	url_guess.searchParams.append("lang", "fr");
	let guess_response = await fetch(url_guess, options);
	let text_guess_response = await guess_response.text();
	let guess_result = JSON.parse(text_guess_response);
	console.log(guess_result);

	add_row_in_result_table(guess_result);
}

function update_pokemon_list() {
	let element = document.getElementById("pokemon_list");
	for (let i = 0; i < pokemon_names.length; ++i) {
		let new_option = document.createElement("option");
		new_option.value = pokemon_names[i];
		element.appendChild(new_option);
	}
}

async function startup() {
	const options = {
		method: 'GET'
	};

	let pokemon_names_url;
	if (is_local()) {
		pokemon_names_url = new URL("http://localhost:3412/names");
	} else {
		pokemon_names_url = new URL("https://pokedle.baduit.eu/names");
	}
	pokemon_names_url.searchParams.append('lang', 'fr');
	let response = await fetch(pokemon_names_url, options);
	let response_text = await response.text();
	let object_response = JSON.parse(response_text);
	pokemon_names = object_response;
	update_pokemon_list();
}

startup()