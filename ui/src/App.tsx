import { useState, useEffect, Suspense, type ComponentType } from 'react';
import { HashRouter as Router, Routes, Route, NavLink } from 'react-router-dom';
import './App.css';

/**
 * Plugin manifest interface describing plugin metadata
 */
interface PluginManifest {
    id: string;
    frontend: {
        entry: string;       // URL to the plugin's JS bundle
        route: string;       // Route path in host app
        sidebarLabel: string; // Label to show in sidebar
    };
}

/**
 * Convert plugin id to UMD global name
 * E.g., "hello-world" => "helloWorldPlugin"
 */
function getUMDGlobalName(pluginId: string): string {
    return (
        pluginId
            .split('-')
            .map((w, i) => (i === 0 ? w : w[0].toUpperCase() + w.slice(1)))
            .join('') + 'Plugin'
    );
}

/**
 * Dynamically load a plugin script as UMD and return its React component
 * @param plugin Plugin manifest
 */
function loadPluginScript(plugin: PluginManifest): Promise<ComponentType | null> {
    return new Promise((resolve, reject) => {
        const existingScript = document.querySelector(`script[src="${plugin.frontend.entry}"]`);

        if (existingScript) {
            // Script already exists, read component from window
            const Component = (window as any)[getUMDGlobalName(plugin.id)];
            console.log(`[Plugin Loader] Script already loaded for ${plugin.id}`, Component);
            return resolve(Component ?? null);
        }

        // Create new script element
        const script = document.createElement('script');
        script.src = plugin.frontend.entry;
        script.async = true;

        script.onload = () => {
            const Component = (window as any)[getUMDGlobalName(plugin.id)];
            if (!Component) console.warn(`[Plugin Loader] Plugin ${plugin.id} did not attach to window`);
            else console.log(`[Plugin Loader] Plugin ${plugin.id} loaded successfully`, Component);
            resolve(Component ?? null);
        };

        script.onerror = (err) => {
            console.error(`[Plugin Loader] Failed to load plugin script: ${plugin.frontend.entry}`, err);
            reject(err);
        };

        document.body.appendChild(script);
    });
}

/**
 * Main App component
 */
function App() {
    const [plugins, setPlugins] = useState<PluginManifest[]>([]); // List of plugin manifests
    const [pluginModules, setPluginModules] = useState<Record<string, ComponentType>>({}); // Loaded React components

    /**
     * Fetch plugin manifests and dynamically load their scripts
     */
    useEffect(() => {
        const initPlugins = async () => {
            try {
                console.log('[App] Fetching plugin manifests...');
                const res = await fetch('/api/plugins/manifests');
                if (!res.ok) {
                    console.error('Failed to fetch plugin manifests:', res.status, res.statusText);
                    return; // exit early
                }

                const manifests: PluginManifest[] = await res.json();
                console.log('[App] Plugin manifests fetched:', manifests);
                setPlugins(manifests);

                // Load all plugin scripts concurrently
                const loadedModules: Record<string, ComponentType> = {};
                await Promise.all(
                    manifests.map(async (plugin) => {
                        try {
                            const Component = await loadPluginScript(plugin);
                            if (Component) loadedModules[plugin.id] = Component;
                        } catch (err) {
                            console.error(`[App] Error loading plugin ${plugin.id}:`, err);
                        }
                    })
                );

                setPluginModules(loadedModules);
                console.log('[App] Loaded plugin modules:', loadedModules);
            } catch (err) {
                console.error('[App] Error initializing plugins:', err);
            }
        };

        initPlugins().catch((err) => console.error('[App] Unexpected error in initPlugins:', err));
    }, []);

    return (
        <Router>
            <div className="app-container">
                {/* Sidebar navigation */}
                <nav className="sidebar">
                    <div className="sidebar-header">
                        <h3>ReAuth</h3>
                    </div>
                    <ul className="nav-list">
                        <li>
                            <NavLink to="/" end>
                                Home
                            </NavLink>
                        </li>
                    </ul>

                    {plugins.length > 0 && (
                        <>
                            <hr />
                            <div className="plugins-header">
                                <h4>Plugins</h4>
                            </div>
                            <ul className="nav-list">
                                {plugins.map((plugin) => (
                                    <li key={plugin.id}>
                                        <NavLink to={plugin.frontend.route} end>
                                            {plugin.frontend.sidebarLabel}
                                        </NavLink>
                                    </li>
                                ))}
                            </ul>
                        </>
                    )}
                </nav>

                {/* Main content area */}
                <main className="main-content">
                    <Suspense fallback={<div>Loading plugin...</div>}>
                        <Routes>
                            {/* Default home route */}
                            <Route
                                path="/"
                                element={
                                    <div>
                                        <h1>Welcome to ReAuth Core</h1>
                                        <p>Select a plugin from the sidebar.</p>
                                    </div>
                                }
                            />

                            {/* Plugin routes */}
                            {plugins.map((plugin) => {
                                const Component = pluginModules[plugin.id];
                                if (!Component) {
                                    console.warn(`[App] Component not loaded yet for plugin ${plugin.id}`);
                                    return null;
                                }

                                const routePath = plugin.frontend.route.startsWith('/')
                                    ? plugin.frontend.route
                                    : '/' + plugin.frontend.route;

                                console.log(`[App] Adding route for plugin ${plugin.id}:`, routePath);

                                return <Route key={plugin.id} path={routePath} element={<Component />} />;
                            })}

                            {/* Fallback route */}
                            <Route path="*" element={<div>Page Not Found</div>} />
                        </Routes>
                    </Suspense>
                </main>
            </div>
        </Router>
    );
}

export default App;
