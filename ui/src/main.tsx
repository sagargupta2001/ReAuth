import React, { StrictMode } from 'react';
import * as ReactDOMClient from 'react-dom/client';
import * as ReactDOM from 'react-dom';
import * as jsxRuntime from 'react/jsx-runtime';

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

// --- Standard app mount ---
const root = ReactDOMClient.createRoot(document.getElementById('root')!);
root.render(
    <StrictMode>
        <App />
    </StrictMode>
);
