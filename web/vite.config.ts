import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

// https://vite.dev/config/
export default defineConfig({
  build: {
    outDir: "../dist",
    emptyOutDir: true,
  },
  plugins: [tailwindcss(), react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    proxy: {
      "/growtopia-cdn": {
        target: "https://growserver-cache.netlify.app",
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/growtopia-cdn/, ""),
      },
    },
  },
});
