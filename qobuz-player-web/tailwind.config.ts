import type { Config } from "tailwindcss";
import safeArea from "tailwindcss-safe-area";

export default {
  content: ["src/**/*.{rs,js,ts}"],
  plugins: [safeArea],
  future: {
    hoverOnlyWhenSupported: true,
  },
} satisfies Config;
