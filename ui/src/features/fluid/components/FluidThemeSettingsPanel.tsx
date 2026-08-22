import { useCallback, useMemo } from 'react'

import type { ThemeAsset } from '@/entities/theme/model/types'
import { ThemeSettingsSection } from '@/features/fluid/components/settings/ThemeSettingsSection'
import {
  ThemeSettingsProvider,
  type ThemeSettingsContextValue,
} from '@/features/fluid/components/settings/themeSettingsContext'
import {
  readLayoutShell,
  withLayoutShell,
  withTokenValue,
} from '@/features/fluid/lib/tokenAccess'
import { THEME_SETTINGS_SECTIONS } from '@/features/fluid/model/themeSettingsSchema'
import type { TokenGroup } from '@/features/fluid/model/tokens'

interface FluidThemeSettingsPanelProps {
  tokens: Record<string, unknown>
  onTokensChange: (tokens: Record<string, unknown>) => void
  layout: Record<string, unknown>
  onLayoutChange: (layout: Record<string, unknown>) => void
  assets: ThemeAsset[]
  onUploadAsset: (file: File) => void
  isUploading?: boolean
}

/**
 * Left sidebar for theme-wide tokens, layout and assets.
 *
 * The panel itself holds no per-field knowledge — it renders
 * `THEME_SETTINGS_SECTIONS` and provides the read/write plumbing fields need.
 */
export function FluidThemeSettingsPanel({
  tokens,
  onTokensChange,
  layout,
  onLayoutChange,
  assets,
  onUploadAsset,
  isUploading = false,
}: FluidThemeSettingsPanelProps) {
  const setToken = useCallback(
    (group: TokenGroup, token: string, value: unknown) => {
      onTokensChange(withTokenValue(tokens, group, token, value))
    },
    [tokens, onTokensChange],
  )

  const onLayoutShellChange = useCallback(
    (shell: string) => {
      onLayoutChange(withLayoutShell(layout, shell))
    },
    [layout, onLayoutChange],
  )

  const settingsContext: ThemeSettingsContextValue = useMemo(
    () => ({
      tokens,
      setToken,
      layoutShell: readLayoutShell(layout),
      onLayoutShellChange,
      assets,
      onUploadAsset,
      isUploading,
    }),
    [tokens, setToken, layout, onLayoutShellChange, assets, onUploadAsset, isUploading],
  )

  return (
    <ThemeSettingsProvider value={settingsContext}>
      <aside className="bg-muted/10 flex w-72 flex-col border-r">
        <div className="bg-background border-b px-4 py-5">
          <h3 className="text-sm font-semibold">Theme Settings</h3>
        </div>

        <div className="flex-1 space-y-4 overflow-y-auto p-4">
          {THEME_SETTINGS_SECTIONS.map((section) => (
            <ThemeSettingsSection key={section.id} section={section} />
          ))}
        </div>
      </aside>
    </ThemeSettingsProvider>
  )
}
