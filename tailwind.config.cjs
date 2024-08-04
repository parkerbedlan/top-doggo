import typography from "@tailwindcss/typography";
import forms from "@tailwindcss/forms";

/** @type {import('tailwindcss').Config} */
const config = {
  content: ["./templates/**/*.html", "./src/**/*.rs"],
  theme: {
    extend: {
      fontFamily: {
        shantell: ['"Shantell Sans"', "sans-serif"],
      },
      keyframes: {
        "drop-in": {
          from: {
            transform: "translateY(-15px)",
            opacity: "0",
          },
          to: {
            top: "translateY(0)",
            opacity: "1",
          },
        },
        "scale-up-down": {
          "0%": { transform: "scale(0)" },
          "25%": { transform: "scale(0)" },
          "75%": {
            transform: "scale(1.2)",
            animationTimingFunction: "ease-in-out",
          },
          "100%": {
            transform: "scale(1)",
            animationTimingFunction: "ease-in-out",
          },
        },
      },
      animation: {
        "drop-in": "drop-in .4s 1",
        "scale-up-down": "scale-up-down 2s ease-in-out 1",
      },
    },
  },
  plugins: [require("daisyui"), forms({ strategy: "class" }), typography],
  daisyui: {
    themes: [
      {
        light: {
          ...require("daisyui/src/theming/themes")["light"],
          primary: "#9333ea",
          secondary: "#eab308",
          accent: "#22c55e",
        },
      },
      {
        dark: {
          ...require("daisyui/src/theming/themes")["dark"],
          primary: "#9333ea",
          secondary: "#eab308",
          accent: "#22c55e",
        },
      },
    ],
  },
};
module.exports = config;
