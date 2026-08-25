import type { ReactNode } from 'react'

import { cn } from '@/lib/utils'

/**
 * The page shell a Fluid theme's `layout.shell` selects.
 *
 * Both renderers used to carry their own copy of this markup, and the runtime
 * carried it twice — once per shell branch — so the error banner and the
 * developer warning were written out identically in both. Changing the shell
 * meant four edits, which is how the `SplitScreen` brand pane and the
 * `CenteredCard` surface drifted apart in the first place.
 *
 * The shell owns chrome only. What goes inside it — an inert preview form or a
 * live one wired to react-hook-form — is the caller's `children`.
 */
export interface FluidShellProps {
  /** `layout.shell`: `SplitScreen`, `Minimal`, or the default centred card. */
  shell: string
  surface: string
  background: string
  text: string
  radiusBase: number
  /** Blocks for the `SplitScreen` brand pane. Ignored by the other shells. */
  brand?: ReactNode
  /** True when there are no brand blocks, so the pane shows its placeholder. */
  hasBrand?: boolean
  children: ReactNode
}

export function FluidShell({
  shell,
  surface,
  background,
  text,
  radiusBase,
  brand,
  hasBrand = false,
  children,
}: FluidShellProps) {
  if (shell === 'SplitScreen') {
    return (
      <div
        className="grid w-full max-w-4xl grid-cols-1 overflow-hidden rounded-2xl border shadow-lg md:grid-cols-2"
        style={{ backgroundColor: surface }}
      >
        <div className="flex flex-col justify-between bg-slate-900 p-8 text-white">
          {hasBrand ? (
            <div className="space-y-3">{brand}</div>
          ) : (
            <div className="space-y-2 text-xs text-white/60">
              <div className="h-3 w-16 rounded-full bg-white/40" />
              <div className="h-2 w-24 rounded-full bg-white/20" />
              <p>Add brand blocks in Fluid.</p>
            </div>
          )}
          <div className="h-24 rounded-xl border border-white/10 bg-white/5" />
        </div>
        <div className="p-8" style={{ backgroundColor: background, color: text }}>
          {children}
        </div>
      </div>
    )
  }

  return (
    <div
      className={cn(
        'w-full max-w-md border p-8',
        shell === 'Minimal' ? 'border-transparent shadow-none' : 'shadow-lg',
      )}
      style={{
        borderRadius: `${radiusBase}px`,
        backgroundColor: shell === 'Minimal' ? 'transparent' : surface,
        color: text,
      }}
    >
      {children}
    </div>
  )
}
