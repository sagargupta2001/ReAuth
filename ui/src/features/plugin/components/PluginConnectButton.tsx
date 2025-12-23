import { Button } from '@/components/button'
import { usePluginMutations } from '@/features/plugin/api/usePluginMutations'
import type { PluginStatusInfo } from '@/entities/plugin/model/types'

interface Props {
  plugin: PluginStatusInfo
}

export function PluginConnectButton({ plugin }: Props) {
  const { enablePlugin, disablePlugin } = usePluginMutations()
  const isLoading = enablePlugin.isPending || disablePlugin.isPending

  if (plugin.status === 'active') {
    return (
      <Button
        variant="outline"
        size="sm"
        disabled={isLoading}
        onClick={() => disablePlugin.mutate(plugin.manifest.id)}
        className="border-red-300 bg-red-50 hover:bg-red-100 dark:border-red-700 dark:bg-red-950 dark:hover:bg-red-900"
      >
        {isLoading ? 'Disabling...' : 'Disable'}
      </Button>
    )
  }

  if (plugin.status === 'inactive') {
    return (
      <Button
        variant="outline"
        size="sm"
        disabled={isLoading}
        onClick={() => enablePlugin.mutate(plugin.manifest.id)}
        className="border-blue-300 bg-blue-50 hover:bg-blue-100 dark:border-blue-700 dark:bg-blue-950 dark:hover:bg-blue-900"
      >
        {isLoading ? 'Enabling...' : 'Enable'}
      </Button>
    )
  }

  return (
    <Button variant="destructive" size="sm" disabled>
      Failed
    </Button>
  )
}
