const rust = import("../frontend/pkg/editor_frontend.js");

rust
	.then((m) => {
		console.log("god help us please");
		m.greet();
	})
	.catch(console.error);

let canvas = document.querySelector("#frame");
canvas.height = 340;
canvas.width = 340 / 9 * 16;

let cx = canvas.getContext("2d");

// foo(function(tile) {
	
// });

window.rSocket = new WebSocket("ws://127.0.0.1:3012");

rSocket.onmessage = (e) => {
	let tile = JSON.parse(e.data);

	for (let i = 0; i < tile.width; i++) {
		for (let b = 0; b < tile.height; b++) {
			let cr = tile.data[i + b * tile.width].x / tile.sample_count;
			let cg = tile.data[i + b * tile.width].y / tile.sample_count;
			let cb = tile.data[i + b * tile.width].z / tile.sample_count;
			
			let exposure = 1.0;
			let gamma = 2.2;

			// Tone mapping
			cr = 1.0 - Math.exp(cr * -1.0 * exposure);
			cg = 1.0 - Math.exp(cg * -1.0 * exposure);
			cb = 1.0 - Math.exp(cb * -1.0 * exposure);

			// Game correct
			cr = Math.pow(cr, 1.0 / gamma);
			cg = Math.pow(cg, 1.0 / gamma);
			cb = Math.pow(cb, 1.0 / gamma);

			cx.fillStyle = "rgb(" +
				Math.floor(cr * 255) + "," +
				Math.floor(cg * 255) + "," +
				Math.floor(cb * 255) + ")";
			

			cx.fillRect(i + tile.left, b + tile.top, 1, 1);
		}
	}
};

rSocket.onclose = (e) => {
	console.log("Socket closed");
};

rSocket.onerror = (e) => {
	console.log("Socket error: " + e);
};