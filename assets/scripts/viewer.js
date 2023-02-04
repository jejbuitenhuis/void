(async function(){
	/**
	 * @param {string} row
	 * @returns {string}
	 */
	function parseRow(row, rowNumber) {
		return `<tr><td class="row-number">${rowNumber}</td><td class="row-content">${row}</td></tr>`;
	}

	console.log("Viewer loaded!");

	const ELEM = document.getElementById("content");
	const content = ELEM.textContent.split("\n");
	let parsedContent = [ "<table>" ];

	for (let i = 0; i < content.length; i++)
		parsedContent.push( parseRow( content[i], i + 1 ) );

	parsedContent.push("</table>");

	ELEM.innerHTML = parsedContent.join("");
})();
