import type { ComponentType } from 'react';
import type { PluginManifest } from '../types';

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
export function loadPluginScript(plugin: PluginManifest): Promise<ComponentType | null> {
    return new Promise((resolve, reject) => {
        const existingScript = document.querySelector(`script[src="${plugin.frontend.entry}"]`);
        const globalName = getUMDGlobalName(plugin.id);

        if (existingScript) {
            const Component = (window as any)[globalName];
            console.log(`[Plugin Loader] Script already loaded for ${plugin.id}`, Component);
            return resolve(Component ?? null);
        }

        const script = document.createElement('script');
        script.src = plugin.frontend.entry;
        script.async = true;

        script.onload = () => {
            const Component = (window as any)[globalName];
            if (!Component) console.warn(`[Plugin Loader] Plugin ${plugin.id} did not attach to window.${globalName}`);
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