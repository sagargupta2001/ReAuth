import { CheckIcon, PlusCircledIcon } from '@radix-ui/react-icons'
import { SearchIcon } from 'lucide-react'

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
import { Input } from '@/components/input'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/popover'
import { Separator } from '@/components/separator'
import type { UserRoleFilter } from '@/features/user/api/useUserRoles'
import { userRoleFilterOptions } from '@/features/user/model/userRoleFilters'
import { cn } from '@/lib/utils'

interface UserRolesToolbarProps {
  searchValue: string
  onSearchChange: (value: string) => void
  filterValue: UserRoleFilter
  onFilterChange: (value: UserRoleFilter) => void
}

export function UserRolesToolbar({
  searchValue,
  onSearchChange,
  filterValue,
  onFilterChange,
}: UserRolesToolbarProps) {
  const selectedFilter =
    userRoleFilterOptions.find((option) => option.value === filterValue) ?? userRoleFilterOptions[0]

  return (
    <div className="flex min-w-0 flex-col gap-3 px-1 pt-1">
      <p className="text-muted-foreground min-w-0 text-sm">
        Direct roles are assigned here. Effective roles include group and composite access.
      </p>

      <div className="flex min-w-0 flex-col items-stretch gap-2 sm:flex-row sm:items-center">
        <div className="relative min-w-0 sm:w-64 md:w-72">
          <SearchIcon
            aria-hidden="true"
            className="text-muted-foreground absolute top-1/2 left-4 -translate-y-1/2"
            size={16}
          />
          <Input
            placeholder="Search..."
            value={searchValue}
            onChange={(event) => onSearchChange(event.target.value)}
            className="h-8 w-full min-w-0 pl-10"
          />
        </div>

        <Popover>
          <PopoverTrigger asChild>
            <Button
              variant="outline"
              size="sm"
              className="h-8 shrink-0 justify-start border-dashed sm:justify-center"
              aria-label="Filter roles by assignment"
            >
              <PlusCircledIcon className="size-4" />
              Access
              {filterValue !== 'all' ? (
                <>
                  <Separator orientation="vertical" className="mx-1 h-4" />
                  <Badge variant="secondary" className="rounded-sm px-1 font-normal">
                    {selectedFilter.label}
                  </Badge>
                </>
              ) : null}
            </Button>
          </PopoverTrigger>
          <PopoverContent className="w-[220px] p-0" align="start">
            <Command>
              <CommandInput placeholder="Access" />
              <CommandList>
                <CommandEmpty>No filters found.</CommandEmpty>
                <CommandGroup>
                  {userRoleFilterOptions.map((option) => {
                    const isSelected = filterValue === option.value
                    return (
                      <CommandItem
                        key={option.value}
                        onSelect={() => onFilterChange(option.value)}
                      >
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
      </div>
    </div>
  )
}
