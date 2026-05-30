/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        background: "var(--background)",
        panel: "var(--panel)",
        "panel-2": "var(--panel-2)",
        parchment: "var(--parchment)",
        hairline: "var(--hairline)",
        ink: "var(--ink)",
        muted: "var(--muted)",
        "accent-source": "var(--accent-source)",
        "accent-human": "var(--accent-human)",
        "accent-verified": "var(--accent-verified)",
        "accent-warning": "var(--accent-warning)",
      },
      // Three voices (Spec §9). Named fonts first, safe fallbacks after.
      // No Inter (AI-slop marker). woff2 self-hosting is a follow-up if needed.
      fontFamily: {
        serif: ['"Newsreader"', "Georgia", '"Times New Roman"', "serif"],
        sans: ['"Public Sans"', "system-ui", "-apple-system", '"Segoe UI"', "Roboto", "sans-serif"],
        mono: ['"IBM Plex Mono"', "ui-monospace", '"Cascadia Code"', "Consolas", "monospace"],
      },
    },
  },
  plugins: [],
};
