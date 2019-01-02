// Modules to control application life and create native browser window
const { app, BrowserWindow } = require('electron')
const net = require("net");

// Keep a global reference of the window object, if you don't, the window will
// be closed automatically when the JavaScript object is garbage collected.
let mainWindow

function createWindow() {
	console.log(process.versions);

	// Create the browser window.
	mainWindow = new BrowserWindow({ width: 1400, height: 900 })

	// and load the index.html of the app.
	mainWindow.loadFile('dist/index.html')

	// Open the DevTools.
	// mainWindow.webContents.openDevTools()

	// Emitted when the window is closed.
	mainWindow.on('closed', function () {
		// Dereference the window object, usually you would store windows
		// in an array if your app supports multi windows, this is the time
		// when you should delete the corresponding element.
		mainWindow = null
	});

	const HOST = "127.0.0.1";
	const PORT = 17025;

	let client = new net.Socket();
	client.connect(PORT, HOST, () => {
		console.log("Connected to server.");
		client.write('{"type":"TileProgressed","data":{"sample_count":15,"width":12,"height":18,"left":15,"top":42,"data":[]}}\r\n');
	});
	
	client.write('{"message":"cyka blyat"}\r\n');
	setTimeout(() => client.write('{"message": "..."}'), 500);

	client.on("data", (data) => {
		console.log("DATA: ", data);
	});
	
	client.on("close", () => {
		console.log("Connection closed")
	})

	client.on("error", (err) => {
		console.log(err);
	})

	mainWindow.webContents.openDevTools();
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.on('ready', createWindow)

// Quit when all windows are closed.
app.on('window-all-closed', function () {
	// On OS X it is common for applications and their menu bar
	// to stay active until the user quits explicitly with Cmd + Q
	if (process.platform !== 'darwin') {
		app.quit()
	}
})

app.on('activate', function () {
	// On OS X it's common to re-create a window in the app when the
	// dock icon is clicked and there are no other windows open.
	if (mainWindow === null) {
		createWindow()
	}
})

// In this file you can include the rest of your app's specific main process
// code. You can also put them in separate files and require them here.

