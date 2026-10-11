/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        spotlight: {
          accent: "var(--accent-color)",
          surface: "var(--surface-color)",
          text: "var(--primary-text)",
          muted: "var(--secondary-text)",
        },
      },
      borderRadius: {
        spotlight: "var(--panel-radius)",
      },
      boxShadow: {
        spotlight: "var(--panel-shadow)",
      },
    },
  },
  plugins: [],
};
