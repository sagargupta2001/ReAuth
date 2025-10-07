import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// --- ADD THESE THREE LINES ---
// @ts-ignore
import path from 'path';
import { fileURLToPath } from 'url';
// @ts-ignore
const __dirname = path.dirname(fileURLToPath(import.meta.url));
// -----------------------------

export default defineConfig({
    plugins: [react()],
    resolve: {
        alias: {
            // This will now work correctly
            'react': path.resolve(__dirname, '../../../ui/node_modules/react'),
            'react-dom': path.resolve(__dirname, '../../../ui/node_modules/react-dom'),
        },
    },
    build: {
        lib: {
            entry: 'src/main.tsx',
            formats: ['es'],
            fileName: () => 'module.js',
        },
        rollupOptions: {
            external: ['react', 'react-dom'],
        },
    },
    server: {
        port: 5174,
    },
});