import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

const pluginId = 'hello-world'; // or import from package.json/plugin.json

function getUMDName(id: string) {
    return id
        .split('-')
        .map(w => w[0].toUpperCase() + w.slice(1))
        .join('') + 'Plugin';
}

export default defineConfig({
    plugins: [react()],
    build: {
        lib: {
            entry: 'src/main.tsx',
            formats: ['umd'],
            name: getUMDName(pluginId), // this now matches the host computation
            fileName: () => 'module.js',
        },
        rollupOptions: {
            external: ['react', 'react-dom', 'react/jsx-runtime'],
            output: {
                globals: {
                    react: 'React',
                    'react-dom': 'ReactDOM',
                    'react/jsx-runtime': 'jsxRuntime',
                },
            },
        },
    },
});
