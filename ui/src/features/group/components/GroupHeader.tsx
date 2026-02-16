import { MoreVertical, Group as GroupIcon, Trash2 } from 'lucide-react'
import { toast } from 'sonner'

import { Button } from '@/components/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/dropdown-menu'
import { useRealmNavigate } from '@/entities/realm/lib/navigation'
import type { Group } from '@/entities/group/model/types'

interface GroupHeaderProps {
  group: Group
}

export function GroupHeader({ group }: GroupHeaderProps) {
  const navigate = useRealmNavigate()

  const copyId = () => {
    void navigator.clipboard.writeText(group.id)
    toast.success('Group ID copied')
  }

  return (
    <header className="bg-background/95 supports-backdrop-filter:bg-background/60 sticky top-0 z-20 flex h-16 shrink-0 items-center justify-between border-b px-6 backdrop-blur">
      <div className="flex items-center gap-4">
        <div className="bg-primary/10 flex h-10 w-10 items-center justify-center rounded-lg">
          <GroupIcon className="text-primary h-5 w-5" />
        </div>

        <div className="flex flex-col">
          <div className="flex items-center gap-2">
            <h1 className="text-foreground text-lg font-bold tracking-tight">{group.name}</h1>
          </div>
          <div className="text-muted-foreground flex items-center gap-1 text-xs">
            <span>ID:</span>
            <button onClick={copyId} className="hover:text-foreground font-mono hover:underline">
              {group.id.slice(0, 8)}...
            </button>
          </div>
        </div>
      </div>

      <div className="flex items-center gap-3">
        <Button variant="outline" onClick={() => navigate('/groups')} size="sm">
          Back
        </Button>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon">
              <MoreVertical className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem className="text-destructive">
              <Trash2 className="mr-2 h-4 w-4" /> Delete Group
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </header>
  )
}
