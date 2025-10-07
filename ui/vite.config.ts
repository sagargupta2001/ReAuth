import path from "path"
import tailwindcss from "@tailwindcss/vite"
import react from "@vitejs/plugin-react"
import { defineConfig } from "vite"

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    // ADD THIS PROXY CONFIGURATION
    proxy: {
      // Any request from your UI that starts with /api
      // will be forwarded to the backend server.
      '/api': {
        target: 'http://localhost:3000', // Your Rust backend
        changeOrigin: true, // Recommended for virtual hosts
      },
    },
  },
})