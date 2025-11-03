import { Routes, Route } from 'react-router-dom';

import type { PluginManifest, PluginModules } from '../types';

interface AppRoutesProps {
    plugins: PluginManifest[];
    pluginModules: PluginModules;
}

export function AppRoutes({ plugins, pluginModules }: AppRoutesProps) {
    return (
        <Routes>
            {/* Default home route */}
            <Route
                path="/"
                element={
                    <div>
                        <h1 className="text-3xl font-bold">Welcome to ReAuth Core</h1>
                        <p className="text-muted-foreground">Select a plugin from the sidebar.</p>
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
    );
}