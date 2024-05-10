import typography from "@tailwindcss/typography";
import forms from "@tailwindcss/forms";

/** @type {import('tailwindcss').Config} */
const config = {
  content: ["./templates/**/*.html"],
  theme: {
    extend: {},
  },
  plugins: [require("daisyui"), forms({ strategy: "class" }), typography],
};
module.exports = config;
