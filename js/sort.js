// from https://www.w3schools.com/howto/howto_js_sort_table.asp
function sortTable(n) {
	console.log("calling sortTable" + n);
	var table, rows, switching, i, x, y, shouldSwitch, dir, switchcount = 0;
	table = document.getElementById("myTable2");
	switching = true;
	dir = "asc";
	while (switching) {
		switching = false;
		rows = table.rows;
		for (i = 1; i < (rows.length - 1); i++) {
			shouldSwitch = false;
			x = rows[i].getElementsByTagName("TD")[n];
			y = rows[i + 1].getElementsByTagName("TD")[n];
			maybe_x = Math.abs(Number(x.innerHTML));
			x = maybe_x ? maybe_x : x.innerHTML.toLowerCase();
			maybe_y = Math.abs(Number(y.innerHTML));
			y = maybe_y ? maybe_y : y.innerHTML.toLowerCase();
			if (dir == "asc") {
				if (x > y) {
					shouldSwitch = true;
					break;
				}
			} else if (dir == "desc") {
				if (x < y) {
					shouldSwitch = true;
					break;
				}
			}
		}
		if (shouldSwitch) {
			rows[i].parentNode.insertBefore(rows[i + 1], rows[i]);
			switching = true;
			switchcount++;
		} else {
			if (switchcount == 0 && dir == "asc") {
				dir = "desc";
				switching = true;
			}
		}
	}
}
