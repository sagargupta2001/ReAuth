import type { ComponentType } from 'react'

export type PluginStatus = 'inactive' | 'active' | { failed: string }

/**
 * Represents a single plugin entry (manifest + runtime status).
 */
export interface PluginStatusInfo {
  manifest: Manifest
  status: PluginStatus
}

/**
 * Represents the metadata and configuration of a plugin.
 */
export interface Manifest {
  id: string
  name: string
  version: string
  executable: ExecutableTargets
  frontend: FrontendConfig
  events?: PluginEvents
}

/**
 * Maps supported OS/architecture combinations to plugin executable paths.
 */
export interface ExecutableTargets {
  linux_amd64?: string
  windows_amd64?: string
  darwin_amd64?: string
  [key: string]: string | undefined // allows future platform keys
}

/**
 * Defines how the plugin integrates into the frontend.
 */
export interface FrontendConfig {
  entry: string
  route: string
  sidebarLabel: string
}

/**
 * Defines event subscriptions or emitted events for the plugin.
 */
export interface PluginEvents {
  subscribes_to?: string[]
  emits?: string[]
  supported_event_version?: string
}

/**
 * Maps plugin component names to their React component implementations.
 */
export type PluginModules = Record<string, ComponentType>
