const defaultTheme = require("tailwindcss/defaultTheme");
const colors = require("tailwindcss/colors");

module.exports = {
    content: [
        "./src/**/*.rs",
        "./index.html",
        "./src/**/*.html",
        "./src/**/*.css",
    ],
    theme: {
        screens: {
            "2xs": "370px",
            xs: "475px",
            ...defaultTheme.screens,
        },
        colors: {
            primary: "#50587C",
            light: "#6A74A0",
            lighter: "#9BA2BF",
            lightest: "#B4B9CF",
            dark: "#282C3E",
            transparent: colors.transparent,
            neutral: colors.neutral,
            red: colors.red,
            bg: "#FFFFFF",
            bgdark: "#1f2937",
            black: colors.black,
            white: colors.white
        }
   },
    plugins: [],
};
