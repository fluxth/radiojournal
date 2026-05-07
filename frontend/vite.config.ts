import tailwindcss from "@tailwindcss/vite";
import { sveltekit } from "@sveltejs/kit/vite";
import { loadEnv, defineConfig } from "vite";

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, ".", "");
  if (!env.PUBLIC_API_BASE_URL) {
    throw new Error("PUBLIC_API_BASE_URL env var is required");
  }

  return {
    plugins: [tailwindcss(), sveltekit()],
  };
});
