const path = require('path');
const { merge } = require('webpack-merge');
const common = require('./webpack.common.js');

module.exports = merge(common, {
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'bootstrap.js',
    },
    mode: 'development',
    devtool: 'inline-source-map',
    devServer: {
    // contentBase: './dist',
    proxy: {
        '/new_rtc_session': {
            target: 'http://localhost:3030',
            changeOrigin: true,
        },
        '/state': {
            target: 'http://localhost:3030',
            changeOrigin: true,
        },
      // '/ws': {
      //   target: 'http://localhost:3030',
      //   changeOrigin: true,
      //   ws: true,
      // },
  },
},
});
