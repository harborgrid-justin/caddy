/**
 * CADDY v0.3.0 - Webpack Configuration
 * Build configuration for browser extension
 */

const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');

const isDevelopment = process.env.NODE_ENV !== 'production';

module.exports = {
  mode: isDevelopment ? 'development' : 'production',

  // Source maps
  devtool: isDevelopment ? 'inline-source-map' : 'source-map',

  // Entry points
  entry: {
    background: './src/background/index.ts',
    content: './src/content/index.ts',
    popup: './src/popup/index.tsx',
    options: './src/options/index.tsx',
    devtools: './src/devtools/index.ts',
  },

  // Output configuration
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: '[name].js',
    clean: true,
  },

  // Module resolution
  resolve: {
    extensions: ['.ts', '.tsx', '.js', '.jsx', '.json'],
    alias: {
      '@': path.resolve(__dirname, 'src'),
      '@shared': path.resolve(__dirname, 'src/shared'),
      '@background': path.resolve(__dirname, 'src/background'),
      '@content': path.resolve(__dirname, 'src/content'),
      '@popup': path.resolve(__dirname, 'src/popup'),
      '@devtools': path.resolve(__dirname, 'src/devtools'),
      '@options': path.resolve(__dirname, 'src/options'),
    },
  },

  // Module rules
  module: {
    rules: [
      // TypeScript
      {
        test: /\.tsx?$/,
        use: [
          {
            loader: 'ts-loader',
            options: {
              transpileOnly: isDevelopment,
              compilerOptions: {
                noEmit: false,
              },
            },
          },
        ],
        exclude: /node_modules/,
      },

      // CSS
      {
        test: /\.css$/,
        use: [
          isDevelopment ? 'style-loader' : MiniCssExtractPlugin.loader,
          {
            loader: 'css-loader',
            options: {
              sourceMap: true,
            },
          },
        ],
      },

      // Images
      {
        test: /\.(png|jpg|jpeg|gif|svg)$/,
        type: 'asset/resource',
        generator: {
          filename: 'assets/images/[name][ext]',
        },
      },

      // Fonts
      {
        test: /\.(woff|woff2|eot|ttf|otf)$/,
        type: 'asset/resource',
        generator: {
          filename: 'assets/fonts/[name][ext]',
        },
      },
    ],
  },

  // Plugins
  plugins: [
    // Extract CSS
    new MiniCssExtractPlugin({
      filename: '[name].css',
    }),

    // HTML pages
    new HtmlWebpackPlugin({
      template: './src/popup/popup.html',
      filename: 'popup.html',
      chunks: ['popup'],
      inject: 'body',
    }),

    new HtmlWebpackPlugin({
      template: './src/options/options.html',
      filename: 'options.html',
      chunks: ['options'],
      inject: 'body',
    }),

    new HtmlWebpackPlugin({
      template: './src/devtools/devtools.html',
      filename: 'devtools.html',
      chunks: ['devtools'],
      inject: 'body',
    }),

    // Copy static files
    new CopyWebpackPlugin({
      patterns: [
        {
          from: 'manifest.json',
          to: 'manifest.json',
        },
        {
          from: 'icons',
          to: 'icons',
          noErrorOnMissing: true,
        },
        {
          from: 'assets',
          to: 'assets',
          noErrorOnMissing: true,
        },
        {
          from: 'src/content/content.css',
          to: 'content.css',
          noErrorOnMissing: true,
        },
      ],
    }),
  ],

  // Optimization
  optimization: {
    minimize: !isDevelopment,
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        vendor: {
          test: /[\\/]node_modules[\\/]/,
          name: 'vendor',
          priority: 10,
        },
        shared: {
          test: /[\\/]src[\\/]shared[\\/]/,
          name: 'shared',
          priority: 5,
        },
      },
    },
  },

  // Performance
  performance: {
    hints: isDevelopment ? false : 'warning',
    maxEntrypointSize: 512000,
    maxAssetSize: 512000,
  },

  // Stats
  stats: {
    colors: true,
    modules: false,
    children: false,
    chunks: false,
    chunkModules: false,
  },

  // Watch options
  watchOptions: {
    ignored: /node_modules/,
    aggregateTimeout: 300,
    poll: 1000,
  },
};
