import { useMemo, useState } from 'react'

import { Search } from 'lucide-react'

import { Button } from '@/components/button'
import { Input } from '@/components/input'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/popover'
import { ICON_NAMES, renderIcon } from '@/shared/ui/icon-registry'

/** Searchable grid over the icon registry, for the Icon node's `name` prop. */
export function IconPicker({
  value,
  color,
  onSelect,
}: {
  value: string
  color?: string
  onSelect: (next: string) => void
}) {
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')
  const resolvedColor = color && color.trim() ? color : undefined
  const filteredIcons = useMemo(() => {
    const normalized = query.trim().toLowerCase()
    if (!normalized) return ICON_NAMES
    return ICON_NAMES.filter((name) => name.includes(normalized))
  }, [query])

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button variant="outline" size="sm" className="h-9 gap-2 text-xs">
          {renderIcon(value, { size: 14, color: resolvedColor }) ?? (
            <Search className="h-3.5 w-3.5" />
          )}
          <span>{value || 'Browse'}</span>
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[340px] p-3" align="start">
        <div className="space-y-3">
          <div className="relative">
            <Search className="text-muted-foreground/60 absolute left-2.5 top-2.5 h-4 w-4" />
            <Input
              placeholder="Search icons..."
              className="h-8 pl-8 text-xs"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
            />
          </div>
          <div className="grid max-h-48 grid-cols-4 gap-2 overflow-y-auto pr-1">
            {filteredIcons.length === 0 && (
              <div className="text-muted-foreground col-span-4 text-xs">
                No matching icons.
              </div>
            )}
            {filteredIcons.map((name) => (
              <button
                key={name}
                type="button"
                className="hover:bg-muted/40 flex flex-col items-center gap-1 rounded-md px-2 py-2 text-[10px]"
                title={name}
                onClick={() => {
                  onSelect(name)
                  setOpen(false)
                }}
              >
                {renderIcon(name, { size: 16, color: resolvedColor }) ?? (
                  <span className="text-muted-foreground text-[10px]">?</span>
                )}
                <span className="text-muted-foreground truncate">{name}</span>
              </button>
            ))}
          </div>
        </div>
      </PopoverContent>
    </Popover>
  )
}
