import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// Plugin to recover when Vite dev server receives out-of-order requests for
// Svelte virtual CSS modules before their parent .svelte component was transformed.
/** @returns {import('vite').Plugin} */
function svelteVirtualCssRecovery() {
  /** @type {import('vite').ViteDevServer | undefined} */
  let server;
  return {
    name: "svelte-virtual-css-recovery",
    enforce: "pre",
    configureServer(_server) {
      server = _server;
    },
    async load(id) {
      if (typeof id === "string" && id.includes("svelte&type=style")) {
        const [filename] = id.split("?");
        if (server) {
          try {
            await server.transformRequest(filename);
            const sveltePlugin = server.config.plugins.find(
              (p) => p.name === "vite-plugin-svelte"
            );
            if (sveltePlugin && typeof sveltePlugin.load === "function") {
              const res = await sveltePlugin.load.call(this, id);
              if (res) return res;
            }
          } catch {
            // Ignore errors and allow default pipeline to proceed
          }
        }
      }
      return null;
    },
  };
}

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [svelteVirtualCssRecovery(), sveltekit()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: "127.0.0.1",
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
