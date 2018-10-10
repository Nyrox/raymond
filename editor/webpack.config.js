const path = require('path');

const frontend = {
  target: "electron-renderer",
  entry: {
    renderer: "./src/renderer.js"
  },
  output: {
    filename: "[name].js",
    path: path.resolve(__dirname, "dist")
  },
  watch: true
};

const backend = {
  target: "electron-main",
  entry: {
    main: "./src/main.js"
  },
  output: {
    filename: "[name].js",
    path: path.resolve(__dirname, "dist")
  },
  watch: true
};

module.exports=[backend, frontend];
