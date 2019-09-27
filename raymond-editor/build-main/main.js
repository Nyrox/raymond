"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const electron_1 = require("electron");
const constants_1 = require("./modules/constants");
let instance;
electron_1.app.on("window-all-closed", () => {
    if (process.env.NODE_ENV !== "production") {
        instance = createBrowserWindow();
    }
    if (process.platform !== "darwin") {
        electron_1.app.quit();
    }
});
electron_1.app.on("activate", () => {
    if (instance === null) {
        createBrowserWindow();
    }
});
electron_1.app.on("ready", () => {
    createBrowserWindow();
});
function createBrowserWindow() {
    const instance = new electron_1.BrowserWindow({
        title: "Raymond",
        show: true,
        frame: false,
        minWidth: 500,
        minHeight: 400,
        backgroundColor: "#111",
        webPreferences: {
            webSecurity: constants_1.IN_PRODUCTION,
            nodeIntegration: true,
        },
    });
    if (constants_1.IN_PRODUCTION) {
        instance.loadFile("./build/index.html");
    }
    else {
        instance.loadURL(`http://localhost:9000/`);
    }
    instance.webContents.on("crashed", error => {
        console.error(`BrowserWindow crashed:`, error);
    });
    return instance;
}
