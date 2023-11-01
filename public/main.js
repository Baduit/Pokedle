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

function add_row_in_result_table(results) {
	if (results == null) {
		results = ["equal", "equal", "equal", "equal", "equal"];
	}

	let tbodyRef = document.getElementById('result_table').getElementsByTagName('tbody')[0];
	let newRow = tbodyRef.insertRow();
	for (const r of results) {
		let newCell = newRow.insertCell();
		if (r === "equal") {
			newCell.classList.add("equal_result");
			let a_surprise_for_later = document.getElementById("a_surprise_for_later");
			a_surprise_for_later.textContent = "You won !"
		} else if (r === "partially_equal") {
			newCell.classList.add("partially_equal_result");
		} else {
			newCell.classList.add("different_result");
		}
		let text_element = document.createTextNode(r);
		newCell.appendChild(text_element);
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
		url = new URL("http://localhost:3412/guess");
	} else {
		url = new URL("https://pokedle.baduit.eu/guess");
	}

	url.searchParams.append("pokemon_name", pokemon_name);
	url.searchParams.append("lang", "fr");

	let response = await fetch(url, options);
	let text_response = await response.text();
	console.log(text_response);
	let object_response = JSON.parse(text_response);
	console.log(object_response);

	add_row_in_result_table(object_response);
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