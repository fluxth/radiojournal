import { loadEnv, defineConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";
import { sveltekit } from "@sveltejs/kit/vite";
import adapter from "@sveltejs/adapter-static";

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, ".", "");
  if (!env.PUBLIC_API_BASE_URL) {
    throw new Error("PUBLIC_API_BASE_URL env var is required");
  }

  return {
    plugins: [
      tailwindcss(),
      sveltekit({
        // adapter-auto only supports some environments, see https://kit.svelte.dev/docs/adapter-auto for a list.
        // If your environment is not supported or you settled on a specific environment, switch out the adapter.
        // See https://kit.svelte.dev/docs/adapters for more information about adapters.
        adapter: adapter({
          fallback: "_404.html",
        }),
      }),
    ],
  };
});
