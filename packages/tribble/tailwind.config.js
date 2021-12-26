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
   },
    plugins: [],
};
