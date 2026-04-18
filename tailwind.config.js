/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        bg: {
          DEFAULT: "#0e0f11",
          elevated: "#17191c",
          sidebar: "#111214",
          hover: "#1d1f23",
        },
        fg: {
          DEFAULT: "#e6e7ea",
          muted: "#9aa0a6",
          subtle: "#6b7078",
        },
        brand: {
          DEFAULT: "#8ab4f8",
          soft: "#1f2a3e",
        },
        line: "#26282d",
      },
      fontFamily: {
        sans: [
          "Pretendard",
          "-apple-system",
          "BlinkMacSystemFont",
          "Segoe UI",
          "Roboto",
          "Helvetica Neue",
          "sans-serif",
        ],
        mono: [
          "JetBrains Mono",
          "SF Mono",
          "Menlo",
          "Consolas",
          "monospace",
        ],
      },
    },
  },
  plugins: [],
};
