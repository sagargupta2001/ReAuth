import React, { StrictMode } from 'react';
import * as ReactDOMClient from 'react-dom/client';
import * as ReactDOM from 'react-dom';
import * as jsxRuntime from 'react/jsx-runtime';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import App from './App.tsx';

// --- Expose React globals before loading any plugin ---
declare global {
    interface Window {
        React: typeof React;
        ReactDOM: typeof ReactDOM;
        ReactDOMClient: typeof ReactDOMClient;
        jsxRuntime: typeof jsxRuntime;
    }
}

window.React = React;
window.ReactDOM = ReactDOM;
window.ReactDOMClient = ReactDOMClient;
window.jsxRuntime = jsxRuntime;

const queryClient = new QueryClient({
    defaultOptions: {
        queries: {
            // We set staleTime to Infinity because our plugin list won't change
            // unless the user restarts the app, or we add a manual refresh.
            staleTime: Infinity,
            refetchOnWindowFocus: false,
        },
    },
});

// --- Standard app mount ---
const root = ReactDOMClient.createRoot(document.getElementById('root')!);
root.render(
    <StrictMode>
        <QueryClientProvider client={queryClient}>
            <App />
        </QueryClientProvider>
    </StrictMode>
);
