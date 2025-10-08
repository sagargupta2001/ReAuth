import { useState, useEffect, Suspense, type ComponentType } from 'react';
import { HashRouter as Router, Routes, Route, NavLink } from 'react-router-dom';
import './App.css';

interface PluginManifest {
    id: string;
    frontend: {
        entry: string;
        route: string;
        sidebarLabel: string;
    };
}

function getUMDGlobalName(pluginId: string) {
    return pluginId
        .split('-')
        .map((w, i) => (i === 0 ? w : w[0].toUpperCase() + w.slice(1)))
        .join('') + 'Plugin';
}

/**
 * Dynamically load a plugin script (UMD) and return the component attached to window
 */
function loadPluginScript(plugin: PluginManifest): Promise<ComponentType | null> {
    return new Promise((resolve, reject) => {
        // Check if script is already loaded
        const existing = document.querySelector(`script[src="${plugin.frontend.entry}"]`);
        if (existing) {
            const Component = (window as any)[getUMDGlobalName(plugin.id)];
            return resolve(Component ?? null);
        }

        // Create script element
        const script = document.createElement('script');
        script.src = plugin.frontend.entry;
        script.async = true;

        script.onload = () => {
            const Component = (window as any)[getUMDGlobalName(plugin.id)];
            if (!Component) console.warn(`Plugin ${plugin.id} did not attach to window`);
            else console.log(`Plugin ${plugin.id} loaded successfully`);
            resolve(Component ?? null);
        };

        script.onerror = (err) => {
            console.error(`Failed to load plugin script: ${plugin.frontend.entry}`, err);
            reject(err);
        };

        document.body.appendChild(script);
    });
}

function App() {
    const [plugins, setPlugins] = useState<PluginManifest[]>([]);
    const [pluginModules, setPluginModules] = useState<Record<string, ComponentType>>({});

    useEffect(() => {
        const initPlugins = async () => {
            try {
                const res = await fetch('/api/plugins/manifests');
                if (!res.ok) throw new Error(res.statusText);

                const manifests: PluginManifest[] = await res.json();
                setPlugins(manifests);

                // Dynamically load all plugins
                const loadedModules: Record<string, ComponentType> = {};
                await Promise.all(
                    manifests.map(async (p) => {
                        try {
                            const Component = await loadPluginScript(p);
                            if (Component) loadedModules[p.id] = Component;
                        } catch (err) {
                            console.error(`Error loading plugin ${p.id}:`, err);
                        }
                    })
                );

                setPluginModules(loadedModules);
            } catch (err) {
                console.error('Error initializing plugins:', err);
            }
        };

        initPlugins();
    }, []);

    console.log('Loaded pluginModules:', pluginModules);

    return (
        <Router>
            <div className="app-container">
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
                                {plugins.map((p) => (
                                    <li key={p.id}>
                                        <NavLink to={p.frontend.route} end>
                                            {p.frontend.sidebarLabel}
                                        </NavLink>
                                    </li>
                                ))}
                            </ul>
                        </>
                    )}
                </nav>

                <main className="main-content">
                    <Suspense fallback={<div>Loading plugin...</div>}>
                        <Routes>
                            <Route
                                path="/"
                                element={
                                    <div>
                                        <h1>Welcome to ReAuth Core</h1>
                                        <p>Select a plugin from the sidebar.</p>
                                    </div>
                                }
                            />

                            {plugins.map((p) => {
                                const Component = pluginModules[p.id];
                                if (!Component) return null;

                                const routePath = p.frontend.route.startsWith('/')
                                    ? p.frontend.route
                                    : '/' + p.frontend.route;
                                console.log(`Adding route for plugin ${p.id}:`, routePath);

                                return <Route key={p.id} path={routePath} element={<Component />} />;
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
