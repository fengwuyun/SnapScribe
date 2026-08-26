/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        primary: "#4353FF",
        "primary-soft": "#EEF0FF",
        accent: "#FF6B6B",
        "accent-soft": "#FFAAA5",
        bg: "#F7F8FC",
        surface: "#FFFFFF",
        "surface-soft": "#EEF0FF",
        "text-primary": "#111111",
        "text-secondary": "#6F7785",
        "text-tertiary": "#9AA3B2",
        line: "#E5E5EA",
        divider: "#F0F0F5",
        success: "#22A06B",
        warning: "#E59B18",
        error: "#D94F4F",
      },
      borderRadius: {
        sm: "8px",
        md: "12px",
        lg: "16px",
        xl: "24px",
        round: "999px",
      },
      boxShadow: {
        sm: "0 2px 12px rgba(0,0,0,0.06)",
        md: "0 12px 36px rgba(17,17,17,0.10)",
        primary: "0 16px 36px rgba(67,83,255,0.20)",
      },
      fontFamily: {
        sans: [
          '"SF Pro Display"',
          "Inter",
          "-apple-system",
          "BlinkMacSystemFont",
          '"Segoe UI"',
          '"Microsoft YaHei"',
          "Arial",
          "sans-serif",
        ],
        mono: ['"SF Mono"', "Consolas", "monospace"],
      },
      keyframes: {
        "fade-in": {
          from: { opacity: "0" },
          to: { opacity: "1" },
        },
      },
      animation: {
        "fade-in": "fade-in 150ms ease",
      },
    },
  },
  plugins: [],
};
