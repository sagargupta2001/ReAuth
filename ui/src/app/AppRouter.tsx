import {Routes, Route} from 'react-router-dom';
import {usePlugins} from '@/entities/plugin/api/usePlugins';

import {staticRoutes} from './routerConfig';

export function AppRouter() {
    const {data, isLoading} = usePlugins();

    if (isLoading)
        return <div>Loading plugins...</div>;

    const plugins = data?.manifests || [];
    const pluginModules = data?.modules || {};

    return (
        <Routes>
            {/* Render all static routes from the config */}
            {staticRoutes.map(({path, element: Element}) => (
                <Route key={path} path={path} element={<Element/>}/>
            ))}

            {/* Render all dynamic plugin routes */}
            {plugins.map((plugin) => {
                const Component = pluginModules[plugin.id];
                if (!Component) return null;

                const routePath = plugin.frontend.route.startsWith('/')
                    ? plugin.frontend.route
                    : '/' + plugin.frontend.route;

                return <Route key={plugin.id} path={routePath} element={<Component/>}/>;
            })}
        </Routes>
    );
}