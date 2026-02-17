import { useQuery } from '@tanstack/react-query'

import { loadPluginScript } from '../../../entities/plugin/lib/pluginLoader.ts'
import type { PluginModules, PluginStatusInfo } from '../../../entities/plugin/model/types.ts'

const fetchAndLoadPlugins = async () => {
  // Fetch the new status list from the API
  const res = await fetch('/api/plugins/manifests')
  if (!res.ok) {
    throw new Error(`Failed to fetch manifests: ${res.statusText}`)
  }
  const statuses: PluginStatusInfo[] = await res.json()
  console.log('[App] Plugin statuses fetched:', statuses)

  // Load scripts ONLY for active plugins
  const loadedModules: PluginModules = {}
  const activePlugins = statuses.filter((p) => p.status === 'active')

  await Promise.all(
    activePlugins.map(async (plugin) => {
      try {
        const Component = await loadPluginScript(plugin)
        if (Component) loadedModules[plugin.manifest.id] = Component
      } catch (err) {
        console.error(`[App] Error loading plugin ${plugin.manifest.id}:`, err)
      }
    }),
  )

  console.log('[App] Loaded active plugin modules:', loadedModules)

  // Return both the full list of statuses AND the loaded modules
  return { statuses, modules: loadedModules }
}

export function usePlugins() {
  return useQuery({
    queryKey: ['plugins'],
    queryFn: fetchAndLoadPlugins,
    staleTime: Infinity,
    gcTime: Infinity,
  })
}
