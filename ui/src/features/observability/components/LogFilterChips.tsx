import { type ReactNode, useEffect, useState } from 'react'

import { Check, PlusCircle, X } from 'lucide-react'

import { Button } from '@/components/button'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/command'
import { Input } from '@/components/input'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/popover'
import { cn } from '@/lib/utils'

interface ChipShellProps {
  label: string
  display?: string
  open: boolean
  onOpenChange: (open: boolean) => void
  onClear?: () => void
  children: ReactNode
}

/** The always-visible chip: "+ Label" when empty, "Label · value ×" when set. */
function ChipShell({ label, display, open, onOpenChange, onClear, children }: ChipShellProps) {
  const active = !!display

  return (
    <Popover open={open} onOpenChange={onOpenChange}>
      <div
        className={cn(
          'inline-flex h-8 items-center rounded-full border text-xs transition-colors',
          active ? 'bg-muted/40 border-border pr-1 pl-3' : 'border-dashed px-3',
        )}
      >
        <PopoverTrigger asChild>
          <button type="button" className="inline-flex items-center gap-1.5">
            {active ? (
              <>
                <span className="text-muted-foreground">{label}</span>
                <span className="text-foreground max-w-[12rem] truncate font-medium">{display}</span>
              </>
            ) : (
              <>
                <PlusCircle className="size-3.5 opacity-60" />
                <span className="text-muted-foreground">{label}</span>
              </>
            )}
          </button>
        </PopoverTrigger>
        {active && onClear ? (
          <button
            type="button"
            aria-label={`Clear ${label}`}
            onClick={onClear}
            className="hover:bg-muted text-muted-foreground hover:text-foreground ml-1 rounded-full p-1"
          >
            <X className="size-3" />
          </button>
        ) : null}
      </div>
      {children}
    </Popover>
  )
}

interface LogTextFilterChipProps {
  label: string
  value: string
  placeholder?: string
  onApply: (value: string) => void
}

export function LogTextFilterChip({ label, value, placeholder, onApply }: LogTextFilterChipProps) {
  const [open, setOpen] = useState(false)
  const [draft, setDraft] = useState(value)

  useEffect(() => {
    if (open) setDraft(value)
  }, [open, value])

  const commit = () => {
    onApply(draft.trim())
    setOpen(false)
  }

  return (
    <ChipShell
      label={label}
      display={value || undefined}
      open={open}
      onOpenChange={setOpen}
      onClear={() => onApply('')}
    >
      <PopoverContent align="start" className="w-64 p-3">
        <div className="mb-2 text-sm font-medium">Filter by {label}</div>
        <Input
          autoFocus
          value={draft}
          placeholder={placeholder ?? `Enter ${label.toLowerCase()}…`}
          onChange={(event) => setDraft(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Enter') commit()
          }}
          className="h-9"
        />
        <div className="mt-3 flex justify-end gap-2">
          <Button variant="ghost" size="sm" onClick={() => setOpen(false)}>
            Cancel
          </Button>
          <Button size="sm" onClick={commit}>
            Apply
          </Button>
        </div>
      </PopoverContent>
    </ChipShell>
  )
}

interface LogSelectFilterChipProps {
  label: string
  value: string
  options: { value: string; label: string }[]
  searchable?: boolean
  contentClassName?: string
  onChange: (value: string) => void
}

export function LogSelectFilterChip({
  label,
  value,
  options,
  searchable,
  contentClassName,
  onChange,
}: LogSelectFilterChipProps) {
  const [open, setOpen] = useState(false)
  const selected = options.find((option) => option.value === value)

  return (
    <ChipShell
      label={label}
      display={selected?.label}
      open={open}
      onOpenChange={setOpen}
      onClear={() => onChange('')}
    >
      <PopoverContent align="start" className={cn('w-56 p-0', contentClassName)}>
        <Command>
          {searchable ? (
            <CommandInput placeholder={`Search ${label.toLowerCase()}…`} />
          ) : null}
          <CommandList>
            <CommandEmpty>No results found.</CommandEmpty>
            <CommandGroup>
              {options.map((option) => (
                <CommandItem
                  key={option.value}
                  value={option.label}
                  onSelect={() => {
                    onChange(option.value === value ? '' : option.value)
                    setOpen(false)
                  }}
                >
                  <Check
                    className={cn(
                      'size-4 shrink-0',
                      option.value === value ? 'opacity-100' : 'opacity-0',
                    )}
                  />
                  <span className="min-w-0 break-all">{option.label}</span>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </ChipShell>
  )
}
