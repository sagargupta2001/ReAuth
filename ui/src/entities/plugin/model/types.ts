import type { ComponentType } from 'react';

export interface PluginManifest {
    id: string;
    frontend: {
        entry: string;
        route: string;
        sidebarLabel: string;
    };
}

export type PluginModules = Record<string, ComponentType>;