import { app, BrowserWindow, ipcMain } from "electron"

import { IN_PRODUCTION } from "./modules/constants"

let instance: BrowserWindow | null

app.on("window-all-closed", () => {
    if (process.env.NODE_ENV !== "production") {
        instance = createBrowserWindow()
    }

    if (process.platform !== "darwin") {
        app.quit()
    }
})

app.on("activate", () => {
    if (instance === null) {
        createBrowserWindow()
    }
})

app.on("ready", () => {
    createBrowserWindow()
})

function createBrowserWindow() {
    const instance = new BrowserWindow({
      title: "Raymond",
      show: true,
      frame: false,
      minWidth: 500,
      minHeight: 400,
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