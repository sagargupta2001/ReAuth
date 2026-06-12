import { CheckIcon, PlusCircledIcon } from '@radix-ui/react-icons'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/command'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/popover'
import { Separator } from '@/components/separator'
import { cn } from '@/lib/utils'

interface AccessFilterOption<T extends string> {
  value: T
  label: string
}

interface AssignmentAccessFilterProps<T extends string> {
  /** The first option is treated as the "unfiltered" default (no badge shown). */
  options: AccessFilterOption<T>[]
  value: T
  onChange: (value: T) => void
  title?: string
}

/**
 * Compact faceted filter for the data-table toolbar, used to scope role
 * member/composite lists by assignment (All / Direct / Effective / Unassigned).
 * Designed to be passed to `DataTable`'s `toolbarFilters` slot.
 */
export function AssignmentAccessFilter<T extends string>({
  options,
  value,
  onChange,
  title = 'Access',
}: AssignmentAccessFilterProps<T>) {
  const defaultValue = options[0]?.value
  const selected = options.find((option) => option.value === value) ?? options[0]
  const isFiltered = value !== defaultValue

  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          size="sm"
          className="h-8 border-dashed"
          aria-label={`Filter by ${title.toLowerCase()}`}
        >
          <PlusCircledIcon className="size-4" />
          {title}
          {isFiltered ? (
            <>
              <Separator orientation="vertical" className="mx-1 h-4" />
              <Badge variant="secondary" className="rounded-sm px-1 font-normal">
                {selected.label}
              </Badge>
            </>
          ) : null}
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[220px] p-0" align="start">
        <Command>
          <CommandInput placeholder={title} />
          <CommandList>
            <CommandEmpty>No filters found.</CommandEmpty>
            <CommandGroup>
              {options.map((option) => {
                const isSelected = value === option.value
                return (
                  <CommandItem key={option.value} onSelect={() => onChange(option.value)}>
                    <div
                      className={cn(
                        'border-primary flex size-4 items-center justify-center rounded-sm border',
                        isSelected
                          ? 'bg-primary text-primary-foreground'
                          : 'opacity-50 [&_svg]:invisible',
                      )}
                    >
                      <CheckIcon className="h-4 w-4" />
                    </div>
                    <span>{option.label}</span>
                  </CommandItem>
                )
              })}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}
