import type { ReactNode } from 'react'
import {
  ArrowLeft,
  Check,
  ChevronDown,
  CloudUpload,
  Loader2,
  Plus,
  Save,
} from 'lucide-react'
import { useState } from 'react'

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
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/dialog'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/popover'
import { Separator } from '@/components/separator'
import { Input } from '@/components/input'
import type { ThemePageTemplate } from '@/entities/theme/model/types'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'

interface FluidBuilderHeaderProps {
  themeName: string
  pages: ThemePageTemplate[]
  activePageKey: string
  onSelectPage: (pageKey: string) => void
  onCreatePage?: (label: string) => void
  onSave: () => void
  onResetPage?: () => void
  onPublish: () => void
  actions?: ReactNode
  isSaving?: boolean
  isPublishing?: boolean
  canResetPage?: boolean
}

export function FluidBuilderHeader({
  themeName,
  pages,
  activePageKey,
  onSelectPage,
  onCreatePage,
  onSave,
  onResetPage,
  onPublish,
  actions,
  isSaving,
  isPublishing,
  canResetPage = false,
}: FluidBuilderHeaderProps) {
  const navigate = useRealmNavigate()
  const isBusy = Boolean(isSaving || isPublishing)
  const activePage = pages.find((page) => page.key === activePageKey)
  const [isPageOpen, setIsPageOpen] = useState(false)
  const [isCreateOpen, setIsCreateOpen] = useState(false)
  const [newPageName, setNewPageName] = useState('')

  const handleCreate = () => {
    const trimmed = newPageName.trim()
    if (!trimmed || !onCreatePage) return
    onCreatePage(trimmed)
    setNewPageName('')
    setIsCreateOpen(false)
  }

  return (
    <header className="bg-muted/20 flex h-14 shrink-0 items-center justify-between border-b px-4">
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="sm" onClick={() => navigate(-1)}>
          <ArrowLeft className="mr-2 h-4 w-4" />
          Exit
        </Button>
        <Separator orientation="vertical" className="h-6" />

        <div className="flex flex-col">
          <div className="flex items-center gap-2">
            <span className="text-sm font-semibold">{themeName}</span>
            <Badge variant="secondary" className="text-muted-foreground h-4 px-2 text-[9px]">
              Draft
            </Badge>
          </div>
          <span className="text-muted-foreground text-[10px] tracking-wider uppercase">
            Fluid Theme Builder
          </span>
        </div>
      </div>

      <div className="flex flex-1 items-center justify-center">
        <Popover open={isPageOpen} onOpenChange={setIsPageOpen}>
          <PopoverTrigger asChild>
            <Button variant="outline" size="sm" className="gap-2">
              <span className="text-xs font-semibold">
                {activePage?.label ?? 'Select Page'}
              </span>
              <ChevronDown className="h-3.5 w-3.5 text-muted-foreground" />
            </Button>
          </PopoverTrigger>
          <PopoverContent align="center" className="w-64 p-0">
            <Command>
              <CommandInput placeholder="Search pages..." />
              <CommandList>
                <CommandEmpty>No pages found.</CommandEmpty>
                <CommandGroup>
                  {pages.map((page) => (
                    <CommandItem
                      key={page.key}
                      onSelect={() => {
                        onSelectPage(page.key)
                        setIsPageOpen(false)
                      }}
                    >
                      <span className="flex flex-1 flex-col">
                        <span className="text-xs font-medium">{page.label}</span>
                        <span className="text-muted-foreground text-[10px]">
                          {page.description}
                        </span>
                      </span>
                      {page.key === activePageKey && (
                        <Check className="h-3.5 w-3.5 text-primary" />
                      )}
                    </CommandItem>
                  ))}
                </CommandGroup>
                {onCreatePage && (
                  <CommandGroup>
                    <CommandItem
                      onSelect={() => {
                        setIsPageOpen(false)
                        setIsCreateOpen(true)
                      }}
                      className="text-primary flex items-center gap-2"
                    >
                      <Plus className="h-4 w-4" />
                      <span className="text-xs font-semibold">Create new page</span>
                    </CommandItem>
                  </CommandGroup>
                )}
              </CommandList>
            </Command>
          </PopoverContent>
        </Popover>
      </div>

      <div className="flex items-center gap-2">
        {actions}
        {onResetPage && (
          <Button
            variant="outline"
            size="sm"
            onClick={onResetPage}
            disabled={!canResetPage}
          >
            Reset Page
          </Button>
        )}
        <Button size="sm" variant="secondary" onClick={onSave} disabled={isBusy}>
          {isSaving ? (
            <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
          ) : (
            <Save className="mr-2 h-3.5 w-3.5" />
          )}
          Save Draft
        </Button>
        <Button size="sm" className="gap-2" onClick={onPublish} disabled={isBusy}>
          {isPublishing ? (
            <Loader2 className="h-3.5 w-3.5 animate-spin" />
          ) : (
            <CloudUpload className="h-3.5 w-3.5" />
          )}
          Publish
        </Button>
      </div>

      <Dialog open={isCreateOpen} onOpenChange={setIsCreateOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Create a new page</DialogTitle>
            <DialogDescription>
              Add a custom page to this theme. You can customize the layout and blocks
              after creating it.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-2">
            <label htmlFor="new-page-name" className="text-xs font-semibold">
              Page name
            </label>
            <Input
              id="new-page-name"
              placeholder="e.g. Welcome"
              value={newPageName}
              onChange={(event) => setNewPageName(event.target.value)}
            />
          </div>
          <DialogFooter className="gap-2 sm:gap-0">
            <Button variant="outline" size="sm" onClick={() => setIsCreateOpen(false)}>
              Cancel
            </Button>
            <Button size="sm" onClick={handleCreate} disabled={!newPageName.trim()}>
              Create page
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </header>
  )
}
