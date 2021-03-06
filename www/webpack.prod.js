const path = require('path');
const { merge } = require('webpack-merge');
// const { BundleAnalyzerPlugin } = require('webpack-bundle-analyzer');
const common = require('./webpack.common.js');

module.exports = merge(common, {
    output: {
        path: path.resolve(__dirname, '..', 'server', 'public'),
        filename: 'bootstrap.js',
    },
    mode: 'production',
    plugins: [],
});
