import { GitBranch, Lock, MoreVertical, Pencil } from 'lucide-react'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/dropdown-menu'
import type { FlowDraft } from '@/entities/flow/model/types'
import { useRealmNavigate } from '@/entities/realm/lib/navigation'

interface FlowHeaderProps {
  draft: FlowDraft
}

export function FlowHeader({ draft }: FlowHeaderProps) {
  const navigate = useRealmNavigate()
  const isSystemFlow = draft.built_in

  return (
    <header className="flex h-16 shrink-0 items-center justify-between border-b px-6">
      <div className="flex items-center gap-4">
        <div className="bg-primary/10 flex h-10 w-10 items-center justify-center rounded-lg">
          {isSystemFlow ? (
            <Lock className="text-primary h-5 w-5" />
          ) : (
            <GitBranch className="text-primary h-5 w-5" />
          )}
        </div>

        <div className="flex flex-col">
          <div className="flex items-center gap-2">
            <h1 className="text-foreground text-lg font-bold tracking-tight">{draft.name}</h1>
            {isSystemFlow ? (
              <Badge variant="secondary" className="h-5 text-[10px]">
                System
              </Badge>
            ) : (
              <Badge variant="outline" className="h-5 text-[10px]">
                Custom
              </Badge>
            )}
          </div>
          <span className="text-muted-foreground text-xs">
            ID: <span className="font-mono opacity-70">{draft.id.slice(0, 8)}...</span>
          </span>
        </div>
      </div>

      <div className="flex items-center gap-3">
        <div className="text-muted-foreground mr-2 flex items-center gap-2 border-r px-3 text-xs">
          {draft.active_version ? (
            <>
              <span className="relative flex h-2 w-2">
                <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-green-400 opacity-75"></span>
                <span className="relative inline-flex h-2 w-2 rounded-full bg-green-500"></span>
              </span>
              Active Version:{' '}
              <span className="text-foreground font-semibold">v{draft.active_version}</span>
            </>
          ) : (
            <>
              <span className="relative flex h-2 w-2">
                <span className="relative inline-flex h-2 w-2 rounded-full bg-yellow-500"></span>
              </span>
              Status: <span className="text-foreground font-semibold">Unpublished Draft</span>
            </>
          )}
        </div>

        <Button onClick={() => navigate(`/flows/${draft.id}/builder`)} className="gap-2">
          <Pencil className="h-3.5 w-3.5" />
          {isSystemFlow ? 'Edit Flow' : 'Edit Draft'}
        </Button>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon">
              <MoreVertical className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem>Duplicate</DropdownMenuItem>
            <DropdownMenuItem className="text-destructive">Delete</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </header>
  )
}
