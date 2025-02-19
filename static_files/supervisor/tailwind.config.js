const defaultTheme = require("tailwindcss/defaultTheme");

/** @type {import('tailwindcss').Config} */
module.exports = {
  presets: [require("@spartan-ng/ui-core/hlm-tailwind-preset")],
  theme: {
    extend: {
      colors: {
        ...defaultTheme.colors,
        "blue-gray": {
          50: "#eceff1",
          100: "#cfd8dc",
          200: "#b0bec5",
          300: "#90a4ae",
          400: "#78909c",
          500: "#607d8b",
          600: "#546e7a",
          700: "#455a64",
          800: "#37474f",
          900: "#263238",
        },
      },
      gridTemplateColumns: {
        720: "repeat(720, minmax(0, 1fr))",
      },
    },
  },
  content: [
    "./src/**/*.{html,ts}",
    "./libs/**/*.{html,ts}",
    "./node_modules/@spartan-ng/**/*.{js,ts,html}", // Add this for external libraries
  ],
};
