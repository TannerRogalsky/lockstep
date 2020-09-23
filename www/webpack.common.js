const path = require('path');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
  entry: './bootstrap.js',
  resolve: {
    extensions: ['.js', '.ts', '.tsx'],
  },
  module: {
    rules: [
      {
        test: /\.ts(x?)$/,
        exclude: /node_modules/,
        use: [
          {
            loader: 'ts-loader',
          },
        ],
      },
      {
        test: /\.css$/,
        use: [
          'style-loader',
          'css-loader',
        ],
      },
      {
        enforce: 'pre',
        test: /\.js$/,
        loader: 'source-map-loader',
      },
    ],
  },
  plugins: [
    new CopyWebpackPlugin([
        'index.html',
    //     {from: '../client/resources', to: 'resources'},
    ]),    
    new WasmPackPlugin({
        crateDirectory: path.resolve(__dirname, '..', 'client'),
        outDir: path.resolve(__dirname, '..', 'pkg'),
        watchDirectories: [
          path.resolve(__dirname, '..', 'shared'),
          path.resolve(__dirname, '..', 'nbody'),
        ],
    }),
  ],
};
