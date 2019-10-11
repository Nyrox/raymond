import { app, BrowserWindow, ipcMain, dialog } from "electron"

import { IN_PRODUCTION } from "./modules/constants"

let instance: BrowserWindow | undefined
let settingsDialog: BrowserWindow | undefined









app.on("window-all-closed", () => {
	if (process.platform !== "darwin") {
		app.quit()
	}
})



function openProject(directory: String) {
	const instance = new BrowserWindow({
		title: "Raymond",
		show: true,
		frame: true,
		minWidth: 1200,
		minHeight: 800,
		backgroundColor: "#111",
		webPreferences: {
			webSecurity: IN_PRODUCTION,
			nodeIntegration: true,
		},
	})

	if (IN_PRODUCTION) {
		instance.loadFile("./build/index.html")
	} else {
		instance.loadURL(`http://localhost:9000/`)
	}

	instance.webContents.on("crashed", error => {
		console.error(`BrowserWindow crashed:`, error)
	})

	return instance
}

app.on("activate", () => {
	if (instance === null) {
	}
})

app.on("ready", async () => {
	const directory = await dialog.showOpenDialog({
		properties: ["openDirectory"]
	});

	if (directory.filePaths && directory.filePaths.length) {
		openProject(directory.filePaths[0])
	} else {
		process.exit(-1)
	}

})