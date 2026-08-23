import { Search } from 'lucide-react'
import { useMemo } from 'react'

import { Input } from '@/components/input'
import {
  filterBlocks,
  groupBlocksByCategory,
  type FluidBlockId,
} from '@/features/fluid/model/blockCatalog'

interface BlockCatalogListProps {
  query: string
  onQueryChange: (query: string) => void
  onHoverBlock: (blockId: FluidBlockId) => void
  onSelectBlock: (blockId: FluidBlockId) => void
}

/** Searchable, category-grouped list of insertable blocks. */
export function BlockCatalogList({
  query,
  onQueryChange,
  onHoverBlock,
  onSelectBlock,
}: BlockCatalogListProps) {
  const groups = useMemo(() => groupBlocksByCategory(filterBlocks(query)), [query])

  return (
    <>
      <div className="relative">
        <Search className="text-muted-foreground/50 absolute left-2.5 top-2.5 h-4 w-4" />
        <Input
          placeholder="Search blocks..."
          aria-label="Search blocks"
          className="bg-background h-9 pl-8 text-sm"
          value={query}
          onChange={(event) => onQueryChange(event.target.value)}
        />
      </div>
      <div className="mt-4 max-h-80 space-y-4 overflow-y-auto pr-1">
        {groups.length === 0 && (
          <p className="text-muted-foreground text-[11px]">No blocks match "{query}".</p>
        )}
        {groups.map(({ category, blocks }) => (
          <div key={category} className="space-y-2">
            <span className="text-muted-foreground text-[10px] font-semibold uppercase tracking-wide">
              {category}
            </span>
            <div className="space-y-1">
              {blocks.map((block) => {
                const Icon = block.icon
                return (
                  <button
                    key={block.id}
                    type="button"
                    onMouseEnter={() => onHoverBlock(block.id)}
                    onFocus={() => onHoverBlock(block.id)}
                    onClick={() => onSelectBlock(block.id)}
                    className="hover:bg-muted/40 flex w-full items-center gap-2 rounded-md px-2 py-2 text-left text-xs transition-colors"
                  >
                    <Icon className="text-muted-foreground h-3.5 w-3.5" />
                    <div className="flex flex-col">
                      <span className="text-[11px] font-semibold">{block.label}</span>
                      <span className="text-muted-foreground text-[10px]">
                        {block.description}
                      </span>
                    </div>
                  </button>
                )
              })}
            </div>
          </div>
        ))}
      </div>
    </>
  )
}
