// src/hooks/usePlugins.ts
import { useState, useEffect } from 'react';
import { loadPluginScript } from '../lib/pluginLoader';
import type { PluginManifest, PluginModules } from '../types';

export function usePlugins() {
    const [plugins, setPlugins] = useState<PluginManifest[]>([]);
    const [pluginModules, setPluginModules] = useState<PluginModules>({});

    useEffect(() => {
        const initPlugins = async () => {
            try {
                console.log('[App] Fetching plugin manifests...');
                const res = await fetch('/api/plugins/manifests');
                if (!res.ok) {
                    console.error('Failed to fetch plugin manifests:', res.status, res.statusText);
                    return;
                }

                const manifests: PluginManifest[] = await res.json();
                console.log('[App] Plugin manifests fetched:', manifests);
                setPlugins(manifests);

                const loadedModules: PluginModules = {};
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

    return { plugins, pluginModules };
}