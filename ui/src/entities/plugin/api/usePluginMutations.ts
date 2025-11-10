import { useMutation, useQueryClient } from '@tanstack/react-query'

const callPluginApi = async (pluginId: string, action: 'enable' | 'disable') => {
  const res = await fetch(`/api/plugins/${pluginId}/${action}`, {
    method: 'POST',
  })
  if (!res.ok) {
    const errorText = await res.text()
    throw new Error(`Failed to ${action} plugin: ${errorText}`)
  }
  return res.json()
}

export function usePluginMutations() {
  const queryClient = useQueryClient()

  // A helper to refetch the plugin list after a change
  const invalidatePlugins = () => {
    void queryClient.invalidateQueries({ queryKey: ['plugins'] })
  }

  const enablePlugin = useMutation({
    mutationFn: (pluginId: string) => callPluginApi(pluginId, 'enable'),
    onSuccess: invalidatePlugins,
  })

  const disablePlugin = useMutation({
    mutationFn: (pluginId: string) => callPluginApi(pluginId, 'disable'),
    onSuccess: invalidatePlugins,
  })

  // The "refresh" button just needs to invalidate the query,
  // which forces `usePlugins` to re-run and call the API again.
  const refreshPlugins = () => {
    invalidatePlugins()
  }

  return {
    enablePlugin,
    disablePlugin,
    refreshPlugins,
  }
}
