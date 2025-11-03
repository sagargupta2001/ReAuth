import { Suspense } from 'react';
import { HashRouter as Router } from 'react-router-dom';

import { Sidebar } from './components/Sidebar';
import { AppRoutes } from './components/AppRoutes';

import { usePlugins } from './hooks/usePlugins';

import './App.css';

/**
 * Main App component
 */
function App() {
    // All async logic is now handled by TanStack Query
    const { data, isLoading, isError } = usePlugins();

    // 1. Handle Loading State
    if (isLoading) {
        return <div className="loading-screen">Loading application...</div>;
    }

    // 2. Handle Error State
    if (isError || !data) {
        return <div className="loading-screen">Error loading application.</div>;
    }

    // 3. Render when data is ready
    const { manifests, modules } = data;

    return (
        <Router>
            <div className="app-container">
                <Sidebar plugins={manifests} />

                <main className="main-content">
                    <Suspense fallback={<div>Loading plugin...</div>}>
                        {/* We'll move the homepage logic back into AppRoutes */}
                        <AppRoutes plugins={manifests} pluginModules={modules} />
                    </Suspense>
                </main>
            </div>
        </Router>
    );
}

export default App;