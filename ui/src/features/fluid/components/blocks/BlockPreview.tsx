import type { ReactElement } from 'react'

import { FluidBlockId } from '@/features/fluid/model/blockCatalog'

/**
 * Miniature rendering of each block, shown beside the picker list.
 *
 * The registry is keyed by `FluidBlockId`, so adding a block to the catalog
 * without adding a preview here is a type error rather than a blank panel.
 */
const BLOCK_PREVIEWS: Record<FluidBlockId, () => ReactElement> = {
  [FluidBlockId.Box]: () => (
    <div className="text-muted-foreground space-y-2 rounded-md border border-dashed p-3 text-[10px]">
      <div className="bg-muted/60 h-3 w-20 rounded-full" />
      <div className="bg-muted/40 h-3 w-24 rounded-full" />
    </div>
  ),
  [FluidBlockId.Text]: () => (
    <div className="text-foreground text-lg font-semibold">Welcome back</div>
  ),
  [FluidBlockId.Input]: () => (
    <div className="space-y-2">
      <span className="text-muted-foreground text-[10px] uppercase tracking-wide">Email</span>
      <div className="bg-muted/20 h-9 w-full rounded-md border" />
    </div>
  ),
  [FluidBlockId.Button]: () => (
    <div className="bg-primary text-primary-foreground h-9 w-full rounded-md text-center text-xs font-semibold">
      Continue
    </div>
  ),
  [FluidBlockId.ProviderButtons]: () => (
    <div className="flex flex-col gap-2">
      <div className="text-primary flex h-9 w-full items-center justify-center rounded-md border text-xs font-medium">
        Continue with Google
      </div>
      <div className="text-primary flex h-9 w-full items-center justify-center rounded-md border text-xs font-medium">
        Continue with Okta
      </div>
    </div>
  ),
  [FluidBlockId.Divider]: () => <div className="bg-border h-px w-full" />,
  [FluidBlockId.Link]: () => <div className="text-primary text-xs underline">Forgot password?</div>,
  [FluidBlockId.Image]: () => (
    <div className="bg-muted/20 text-muted-foreground flex h-28 w-full items-center justify-center rounded-md border text-[10px]">
      Image placeholder
    </div>
  ),
}

export function BlockPreview({ blockId }: { blockId: FluidBlockId }) {
  const render = BLOCK_PREVIEWS[blockId]
  return render ? render() : null
}
