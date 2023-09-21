function doQuery() {
	let rows = document.getElementsByTagName("tr");
	let query = document.getElementById("query").value;
	// skip the header row
	for (let i = 1; i < rows.length; i++) {
		rows[i].style = 'unset';
	}

	if (query === '') {
		return;
	}

	const regex = /\$(\d)/g;
	for (let i = 1; i < rows.length; i++) {
		let cols = rows[i].getElementsByTagName("td");
		let vals = new Array();
		for (let j = 1; j < cols.length; j++) {
			// intentionally indexing from 1 so I can use $1 more
			// easily
			vals[j] = Number(cols[j].innerHTML);
		}
		let expr = query.replace(regex, "vals[$1]");
		if (!eval(expr)) {
			rows[i].style.display = "none";
		}
	}
}
