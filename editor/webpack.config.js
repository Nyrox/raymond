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
  resolve: {
    alias: {
      'vue$': 'vue/dist/vue.esm.js' 
    },
  },
  module: {
    rules: [{
        test: /\.vue$/,
        loader: 'vue-loader'
    }]
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
