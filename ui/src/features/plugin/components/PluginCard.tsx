import { Package } from 'lucide-react'

import type { PluginStatusInfo } from '@/entities/plugin/model/types.ts'
import { RealmLink } from '@/entities/realm/lib/navigation.tsx'
import { PluginConnectButton } from '@/features/plugin/components/PluginConnectButton.tsx'

interface Props {
  plugin: PluginStatusInfo
}

export function PluginCard({ plugin }: Props) {
  return (
    <li className="rounded-lg border p-4 transition-shadow hover:shadow-md">
      <div className="mb-8 flex items-center justify-between">
        <div className="bg-muted flex size-10 items-center justify-center rounded-lg p-2">
          <Package className="text-muted-foreground" />
        </div>
        <PluginConnectButton plugin={plugin} />
      </div>
      <div>
        <RealmLink
          to={plugin.manifest.frontend.route}
          className="underline-offset-4 hover:underline"
        >
          <h2 className="mb-1 font-semibold">{plugin.manifest.name}</h2>
        </RealmLink>
        <p className="text-muted-foreground line-clamp-2 text-sm">
          Version: {plugin.manifest.version}
        </p>
      </div>
    </li>
  )
}
