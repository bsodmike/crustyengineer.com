const defaultTheme = require("tailwindcss/defaultTheme");

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./templates/**/*.html", "./content/**/*.md"],
  theme: {
    extend: {
      colors: {
        // theme: nord
        // https://www.nordtheme.com/docs/colors-and-palettes

        bg: "#212830",
        "bg-light": "#3b4252",

        // nord: Snow storm
        // text: "#e5e9f0",
        text: "#eceff4",

        accent: "#8fbcbb", // primary
        "accent-text": "#2e3440",
        border: "#4c566a",
        link: "#88c0d0", // primary-1

        // nord: frost
        primary: "#8fbcbb",
        "primary-2": "#88c0d0",
        secondary: "#81a1c1",
        terciary: "#5e81ac",
      },
      fontFamily: {
        sans: ['"MPLUS1"', ...defaultTheme.fontFamily.sans],
        mono: ['"MPLUS1Code"', ...defaultTheme.fontFamily.sans],
      },
    },
  },
  plugins: [],
};
