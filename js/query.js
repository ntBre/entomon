function post(api, body) {
	let xhr = new XMLHttpRequest();
	xhr.open("POST", api, false);
	xhr.setRequestHeader('Content-Type', 'application/json');
	xhr.send(body);
}

function makeHandler(expr) {
	return function (data) {
		let toShow = new Array();
		for (let i = 0; i < data.rows.length; i++) {
			if (eval(expr)) {
				toShow.push(i)
			}
		}
		post("/api", JSON.stringify({ Show: toShow }));
		location.reload();
	}
}

function doQuery() {
	let query = document.getElementById("query").value;

	post("/set-query", JSON.stringify(query));

	if (query === '') {
		post("/api", JSON.stringify({ All: null }));
		location.reload();
		return;
	}

	const regex = /\$(\d)/g;
	const pipe = /\|([^|]+)\|/g;
	let expr = query.replace(regex, "data.rows[i].vals[$1 - 1]");
	expr = expr.replace(pipe, "Math.abs($1)");

	fetch("/api").then((response) => {
		return response.json()
	}).then(makeHandler(expr));
}
