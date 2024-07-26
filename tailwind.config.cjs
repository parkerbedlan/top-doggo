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
};
module.exports = config;
