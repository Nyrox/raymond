const path = require("path")
const webpackMerge = require("webpack-merge")

const HtmlPlugin = require("html-webpack-plugin")
const CopyPlugin = require("copy-webpack-plugin")

/** @type {import("webpack").Configuration} */
const baseConfig = {
    mode: "none",
    entry: ["./src/client.tsx"],
    target: "electron-renderer",
    plugins: [
        new HtmlPlugin({
            template: path.resolve(__dirname, "public/index.html"),
            filename: path.resolve(__dirname, "build/public/index.html"),
        }),
    ],
    module: {
        rules: [
            {
                test: /\.[jt]sx?$/,
                use: {
                    loader: "babel-loader",
                    options: {
                        presets: [
                            "@babel/preset-env",
                            "@babel/preset-react",
                            "@babel/preset-typescript",
                        ],
                        plugins: [
                            "transform-class-properties",
                        ],
                    },
                },
            },
        ],
    },
    resolve: {
        extensions: [".tsx", ".ts", ".js"],
    },
    optimization: {
        splitChunks: {
            chunks: "all",
        },
    },
    output: {
        filename: "[name].js?q=[chunkhash]",
        chunkFilename: "[name].js?q=[chunkhash]",
        path: path.resolve(__dirname, "build/public"),
        publicPath: "/",
    },
}

/** @type {import("webpack").Configuration} */
const devConfig = {
    mode: "development",
    devtool: "inline-source-map",
    devServer: {
        contentBase: path.join(__dirname, "public"),
        historyApiFallback: true,
        compress: true,
        port: 9000,
    },
}

/** @type {import("webpack").Configuration} */
const prodConfig = {
    mode: "production",
    plugins: [
        new CopyPlugin([
            {
                from: "./public",
                ignore: ["./public/index.html"],
            },
        ]),
    ],
    optimization: {
        minimize: true,
        nodeEnv: "production",
    },
}

module.exports = (() => {
    if (process.env.NODE_ENV === "production") {
        console.info("Running production config")
        return webpackMerge(baseConfig, prodConfig)
    }

    console.info("Running development config")
    return webpackMerge(baseConfig, devConfig)
})()