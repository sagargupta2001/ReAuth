import { FluidLayoutGallery } from '@/features/fluid/components/FluidLayoutGallery'
import { useThemeSettings } from '@/features/fluid/components/settings/themeSettingsContext'

export function LayoutShellField() {
  const { layoutShell, onLayoutShellChange } = useThemeSettings()
  return <FluidLayoutGallery value={layoutShell} onChange={onLayoutShellChange} />
}
