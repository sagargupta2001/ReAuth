import { useState, useEffect, lazy, Suspense } from 'react';
import { BrowserRouter as Router, Routes, Route, NavLink } from 'react-router-dom';
import './App.css';

// Defines the structure of the data we expect from the backend's plugin.json files.
interface PluginManifest {
    id: string;
    frontend: {
        entry: string;
        route: string;
        sidebarLabel: string;
    };
}

// Helper function to dynamically load a plugin's component using its entry URL.
const loadRemoteComponent = (manifest: PluginManifest) => {
    return lazy(() => {
        // The /* @vite-ignore */ comment is essential. It tells Vite's build tool
        // not to try and resolve this dynamic URL at build time, as it will be
        // provided at runtime by the browser.
        return import(/* @vite-ignore */ manifest.frontend.entry);
    });
};


function App() {
    const [plugins, setPlugins] = useState<PluginManifest[]>([]);

    // This effect runs once when the app starts. It fetches the list of
    // available plugins from the ReAuth core backend.
    useEffect(() => {
        const initPlugins = async () => {
            try {
                const response = await fetch('/api/plugins/manifests');
                if (!response.ok) {
                    console.error("Failed to fetch plugin manifests:", response.statusText);
                    return;
                }
                const manifests: PluginManifest[] = await response.json();
                setPlugins(manifests);
            } catch (error) {
                console.error("Error fetching plugins:", error);
            }
        };
        initPlugins();
    }, []);

    return (
        <Router>
            <div className="app-container">
                <nav className="sidebar">
                    <div className="sidebar-header">
                        <h3>ReAuth</h3>
                    </div>
                    <ul className="nav-list">
                        <li><NavLink to="/">Home</NavLink></li>
                    </ul>

                    {plugins.length > 0 && (
                        <>
                            <hr />
                            <div className="plugins-header">
                                <h4>Plugins</h4>
                            </div>
                            <ul className="nav-list">
                                {/* Dynamically create a navigation link for each loaded plugin */}
                                {plugins.map(p => (
                                    <li key={p.id}>
                                        <NavLink to={p.frontend.route}>{p.frontend.sidebarLabel}</NavLink>
                                    </li>
                                ))}
                            </ul>
                        </>
                    )}
                </nav>

                <main className="main-content">
                    <Suspense fallback={<div className="loading">Loading Page...</div>}>
                        <Routes>
                            <Route path="/" element={
                                <div>
                                    <h1>Welcome to ReAuth Core</h1>
                                    <p>Select a plugin from the sidebar to view its page.</p>
                                </div>
                            } />

                            {/* Dynamically create a route for each plugin */}
                            {plugins.map(p => {
                                const Component = loadRemoteComponent(p);
                                return <Route key={p.id} path={p.frontend.route} element={<Component />} />;
                            })}

                            <Route path="*" element={<div>Page Not Found</div>} />
                        </Routes>
                    </Suspense>
                </main>
            </div>
        </Router>
    );
}

export default App;