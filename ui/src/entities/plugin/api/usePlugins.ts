import { loadPluginScript } from '@/entities/plugin/lib/pluginLoader'
import type { PluginManifest, PluginModules } from '@/entities/plugin/model/types'
import { useSuspenseQuery } from '@/shared/lib/hooks/useSuspenseQuery'

const fetchAndLoadPlugins = async () => {
  // 1. Fetch manifests
  const res = await fetch('/api/plugins/manifests')
  if (!res.ok) throw new Error(`Failed to fetch manifests: ${res.statusText}`)

  const manifests: PluginManifest[] = await res.json()

  // 2. Load all plugin scripts concurrently
  const loadedModules: PluginModules = {}
  await Promise.all(
    manifests.map(async (plugin) => {
      try {
        const Component = await loadPluginScript(plugin)
        if (Component) loadedModules[plugin.id] = Component
      } catch (err) {
        console.error(`[App] Error loading plugin ${plugin.id}:`, err)
      }
    }),
  )

  console.log('[App] Loaded plugin modules:', loadedModules)
  return { manifests, modules: loadedModules }
}

export function usePlugins() {
  return useSuspenseQuery({
    queryKey: ['plugins'],
    queryFn: fetchAndLoadPlugins,
    staleTime: Infinity, // Plugin list won't change in a session
    gcTime: Infinity,
  })
}
