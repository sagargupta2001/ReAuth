import type { ComponentType } from 'react'

import type { PluginStatusInfo } from '../model/types'

function getUMDGlobalName(pluginId: string): string {
  return (
    pluginId
      .split('-')
      .map((w, i) => (i === 0 ? w : w[0].toUpperCase() + w.slice(1)))
      .join('') + 'Plugin'
  )
}

export function loadPluginScript(plugin: PluginStatusInfo): Promise<ComponentType | null> {
  return new Promise((resolve, reject) => {
    const existingScript = document.querySelector(`script[src="${plugin.manifest.frontend.entry}"]`)
    const globalName = getUMDGlobalName(plugin.manifest.id)

    if (existingScript) {
      const Component = (window as any)[globalName]
      console.log(`[Plugin Loader] Script already loaded for ${plugin.manifest.id}`, Component)
      return resolve(Component ?? null)
    }

    const script = document.createElement('script')
    script.src = plugin.manifest.frontend.entry
    script.async = true

    script.onload = () => {
      const Component = (window as any)[globalName]
      if (!Component)
        console.warn(
          `[Plugin Loader] Plugin ${plugin.manifest.id} did not attach to window.${globalName}`,
        )
      else
        console.log(`[Plugin Loader] Plugin ${plugin.manifest.id} loaded successfully`, Component)
      resolve(Component ?? null)
    }

    script.onerror = (err) => {
      console.error(
        `[Plugin Loader] Failed to load plugin script: ${plugin.manifest.frontend.entry}`,
        err,
      )
      reject(err)
    }

    document.body.appendChild(script)
  })
}
