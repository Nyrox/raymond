const path = require('path');

module.exports = {
  entry: './src/renderer.js',
  output: {
    filename: 'renderer.js',
    path: path.resolve(__dirname, 'dist')
  },
  optimization: {
    minimize: false
  }
};
