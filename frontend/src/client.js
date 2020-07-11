let textArea = document.getElementById("editor");
let convertB = document.getElementById("cnvt_button");
let webSock = new WebSocket("ws://localhost:8080");

webSock.onopen = function (event) {
	convertB.addEventListener("click", onConvert);
}

webSock.onmessage = function (event) {
	console.log(event.data);
}

function onConvert() {
	let text = textArea.innerText;
	text = text.replace(/\n\n\n*/g, "\n");
	console.log(text);
	let message = {t: 'u', d: text};
	webSock.send(JSON.stringify(message));
}
