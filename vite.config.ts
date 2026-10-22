import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { fileURLToPath } from "node:url";

// The frontend entry (index.html) lives in src-tauri/ alongside the Tauri
// config, so Vite's root points there. The build output still lands in the
// project-root dist/ to match tauri.conf.json's frontendDist ("../dist").
export default defineConfig({
  root: "src-tauri",
  cacheDir: "../node_modules/.vite",
  plugins: [react()],
  // In dev the browser asks for /src/main.tsx (index.html's ../src/main.tsx
  // seen from the root); point that at the real src/ next to src-tauri/.
  resolve: {
    alias: [{ find: /^\/src\//, replacement: fileURLToPath(new URL("./src/", import.meta.url)) }]
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    outDir: "../dist",
    emptyOutDir: true,
    target: "es2020",
    minify: !process.env.TAURI_DEBUG,
    sourcemap: !!process.env.TAURI_DEBUG
  }
});
