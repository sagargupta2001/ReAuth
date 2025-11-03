// src/App.tsx
import { Suspense } from 'react';
import { HashRouter as Router } from 'react-router-dom';

import { usePlugins } from './hooks/usePlugins';
import { Sidebar } from './components/Sidebar';
import { AppRoutes } from './components/AppRoutes';

import './App.css';

/**
 * Main App component
 */
function App() {
    // All complex logic is now inside this custom hook
    const { plugins, pluginModules } = usePlugins();

    return (
        <Router>
            <div className="app-container">
                {/* Sidebar navigation */}
                <Sidebar plugins={plugins} />

                {/* Main content area */}
                <main className="main-content">
                    <Suspense fallback={<div>Loading plugin...</div>}>
                        <AppRoutes plugins={plugins} pluginModules={pluginModules} />
                    </Suspense>
                </main>
            </div>
        </Router>
    );
}

export default App;