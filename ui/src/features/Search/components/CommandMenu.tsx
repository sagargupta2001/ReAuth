import React from 'react'

import { Laptop, Moon, Sun } from 'lucide-react'

import { useTheme } from '@/app/providers/themeProvider'
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from '@/components/command'
import { ScrollArea } from '@/components/scroll-area'
import { useSearch } from '@/features/Search/model/searchContext'

export function CommandMenu() {
  const { setTheme } = useTheme()
  const { open, setOpen } = useSearch()

  const runCommand = React.useCallback(
    (command: () => unknown) => {
      setOpen(false)
      command()
    },
    [setOpen],
  )

  return (
    <CommandDialog modal open={open} onOpenChange={setOpen}>
      <CommandInput placeholder="Type a command or search..." />
      <CommandList>
        <ScrollArea type="hover" className="h-72 pe-1">
          <CommandEmpty>No results found.</CommandEmpty>
          {/*{sidebarData.navGroups.map((group) => (*/}
          {/*  <CommandGroup key={group.title} heading={group.title}>*/}
          {/*    {group.items.map((navItem, i) => {*/}
          {/*      // Single-level navigation*/}
          {/*      if (navItem.url)*/}
          {/*        return (*/}
          {/*          <CommandItem*/}
          {/*            key={`${navItem.url}-${i}`}*/}
          {/*            value={navItem.title}*/}
          {/*            onSelect={() => runCommand(() => navigate(navItem.url))}*/}
          {/*          >*/}
          {/*            <div className="flex size-4 items-center justify-center">*/}
          {/*              <ArrowRight className="size-2 text-muted-foreground/80" />*/}
          {/*            </div>*/}
          {/*            {navItem.title}*/}
          {/*          </CommandItem>*/}
          {/*        )*/}

          {/*      // Nested navigation*/}
          {/*      return (*/}
          {/*        navItem.items?.map((subItem, j) => (*/}
          {/*          <CommandItem*/}
          {/*            key={`${navItem.title}-${subItem.url}-${j}`}*/}
          {/*            value={`${navItem.title}-${subItem.url}`}*/}
          {/*            onSelect={() => runCommand(() => navigate(subItem.url))}*/}
          {/*          >*/}
          {/*            <div className="flex size-4 items-center justify-center">*/}
          {/*              <ArrowRight className="size-2 text-muted-foreground/80" />*/}
          {/*            </div>*/}
          {/*            {navItem.title} <ChevronRight /> {subItem.title}*/}
          {/*          </CommandItem>*/}
          {/*        )) ?? null*/}
          {/*      )*/}
          {/*    })}*/}
          {/*  </CommandGroup>*/}
          {/*))}*/}

          <CommandSeparator />

          <CommandGroup heading="Theme">
            <CommandItem onSelect={() => runCommand(() => setTheme('light'))}>
              <Sun /> <span>Light</span>
            </CommandItem>
            <CommandItem onSelect={() => runCommand(() => setTheme('dark'))}>
              <Moon className="scale-90" /> <span>Dark</span>
            </CommandItem>
            <CommandItem onSelect={() => runCommand(() => setTheme('system'))}>
              <Laptop /> <span>System</span>
            </CommandItem>
          </CommandGroup>
        </ScrollArea>
      </CommandList>
    </CommandDialog>
  )
}
