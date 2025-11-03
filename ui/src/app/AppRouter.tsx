import { Routes, Route } from 'react-router-dom';
import { usePlugins } from '@/entities/plugin/api/usePlugins';
import { DashboardPage } from '@/pages/DashboardPage';
import { NotFoundPage } from '@/pages/NotFoundPage';

export function AppRouter() {
    const { data, isLoading } = usePlugins();

    if (isLoading) {
        return <div>Loading plugins...</div>;
    }

    const plugins = data?.manifests || [];
    const pluginModules = data?.modules || {};

    return (
        <Routes>
            <Route path="/" element={<DashboardPage />} />

            {/* Dynamically create routes for each loaded plugin */}
            {plugins.map((plugin) => {
                const Component = pluginModules[plugin.id];
                if (!Component) {
                    return null;
                }
                const routePath = plugin.frontend.route.startsWith('/')
                    ? plugin.frontend.route
                    : '/' + plugin.frontend.route;

                return <Route key={plugin.id} path={routePath} element={<Component />} />;
            })}

            <Route path="*" element={<NotFoundPage />} />
        </Routes>
    );
}