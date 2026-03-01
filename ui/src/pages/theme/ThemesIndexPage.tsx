import { Palette } from 'lucide-react'

export function ThemesIndexPage() {
  return (
    <div className="flex h-full flex-col items-center justify-center space-y-4 text-center">
      <div className="bg-muted flex h-20 w-20 items-center justify-center rounded-full">
        <Palette className="text-muted-foreground h-10 w-10" />
      </div>
      <div className="max-w-md space-y-2">
        <h2 className="text-2xl font-bold tracking-tight">Themes</h2>
        <p className="text-muted-foreground">
          Themes control the visual experience for every authenticator node. Select a theme from
          the sidebar to review settings or open it in Fluid for full editing.
        </p>
      </div>
    </div>
  )
}
