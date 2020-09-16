const path = require('path');
const { merge } = require('webpack-merge');
// const { BundleAnalyzerPlugin } = require('webpack-bundle-analyzer');
const common = require('./webpack.common.js');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = merge(common, {
    output: {
        path: path.resolve(__dirname, '..', 'server', 'public'),
        filename: 'bootstrap.js',
    },
    mode: 'production',
    plugins: [
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, '..', 'client'),
            outDir: '../pkg',
        }),
    ],
});
