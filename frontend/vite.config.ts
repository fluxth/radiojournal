import tailwindcss from "@tailwindcss/vite";
import { sveltekit } from "@sveltejs/kit/vite";
import { loadEnv } from "vite";
import { defineConfig } from "vitest/config";

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, ".", "");
  if (!env.PUBLIC_API_BASE_URL) {
    throw new Error("PUBLIC_API_BASE_URL env var is required");
  }

  return {
    plugins: [tailwindcss(), sveltekit()],
    test: {
      include: ["src/**/*.{test,spec}.{js,ts}"],
    },
  };
});
