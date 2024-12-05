import { nextui } from "@nextui-org/react";

/** @type {import('tailwindcss').Config} */
export default {
  content: [
    // ...
    // make sure it's pointing to the ROOT node_module
    "./node_modules/@nextui-org/theme/dist/**/*.{js,ts,jsx,tsx}",
    "./src/**/*.{html,js,tsx,ts}",
  ],
  theme: {
    extend: {},
  },
  plugins: [
    nextui(),
    function ({ addUtilities }) {
      addUtilities({
        ".no-scrollbar": {
          "scrollbar-width": "none", // Firefox
          "-ms-overflow-style": "none", // Internet Explorer 10+
        },
        ".no-scrollbar::-webkit-scrollbar": {
          display: "none", // Chrome, Safari, and Opera
        },
      });
    },
  ],
};
