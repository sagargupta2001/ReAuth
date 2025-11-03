import { useQuery } from '@tanstack/react-query';

import { loadPluginScript } from '../lib/pluginLoader';
import type { PluginManifest, PluginModules } from '../types';

/**
 * Fetches all plugin manifests and dynamically loads their scripts.
 * This entire async block is now managed by TanStack Query.
 */
const fetchAndLoadPlugins = async () => {
    // 1. Fetch manifests
    console.log('[App] Fetching plugin manifests...');
    const res = await fetch('/api/plugins/manifests');
    if (!res.ok) {
        throw new Error(`Failed to fetch manifests: ${res.statusText}`);
    }
    const manifests: PluginManifest[] = await res.json();
    console.log('[App] Plugin manifests fetched:', manifests);

    // 2. Load all plugin scripts concurrently
    const loadedModules: PluginModules = {};
    await Promise.all(
        manifests.map(async (plugin) => {
            try {
                const Component = await loadPluginScript(plugin);
                if (Component) {
                    loadedModules[plugin.id] = Component;
                }
            } catch (err) {
                console.error(`[App] Error loading plugin ${plugin.id}:`, err);
            }
        })
    );

    console.log('[App] Loaded plugin modules:', loadedModules);

    // 3. Return all the data
    return { manifests, modules: loadedModules };
};

/**
 * Custom hook to fetch plugins and their modules, managed by TanStack Query.
 */
export function usePlugins() {
    return useQuery({
        // The queryKey is a unique ID for this data
        queryKey: ['plugins'],

        // The queryFn is the async function that gets the data
        queryFn: fetchAndLoadPlugins,

        // We set `gcTime` (garbage collection time) to Infinity so the
        // plugin data is cached for the entire session.
        gcTime: Infinity,
    });
}