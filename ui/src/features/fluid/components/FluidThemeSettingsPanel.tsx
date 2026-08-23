import { useCallback, useMemo, useState } from 'react'

import { RotateCcw } from 'lucide-react'

import { Button } from '@/components/button'
import { ConfirmDialog } from '@/shared/ui/confirm-dialog'

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
  /**
   * Restores theme-wide settings to the seeded defaults. Omit to hide the action.
   *
   * Distinct from the header's per-page restore: background colour, typography and
   * radius are tokens, not page blocks, so restoring a page cannot revert them.
   *
   * The defaults are fetched by the page, not here — this panel stays
   * presentational so it renders without a query client.
   */
  onResetTokens?: () => void
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
  onResetTokens,
}: FluidThemeSettingsPanelProps) {
  const [isResetOpen, setIsResetOpen] = useState(false)
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
      <aside className="bg-muted/10 flex w-80 flex-col border-r">
        <div className="bg-background flex items-center justify-between border-b px-4 py-3">
          <h3 className="text-sm font-semibold">Theme Settings</h3>
          {onResetTokens && (
            <>
              <Button
                variant="ghost"
                size="sm"
                className="h-7 gap-1.5 px-2 text-xs"
                onClick={() => setIsResetOpen(true)}
              >
                <RotateCcw className="h-3.5 w-3.5" />
                Reset
              </Button>
              <ConfirmDialog
                open={isResetOpen}
                onOpenChange={setIsResetOpen}
                title="Reset theme settings to their defaults?"
                desc="This restores the colours, typography, corner radius, and layout shell that a new theme starts with. Page blocks and uploaded assets are untouched. Like any builder edit it is a draft change, so use Save to persist it."
                confirmText="Reset settings"
                destructive
                handleConfirm={() => {
                  onResetTokens()
                  setIsResetOpen(false)
                }}
              />
            </>
          )}
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
