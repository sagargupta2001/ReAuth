import { Copy, Group as GroupIcon } from 'lucide-react'
import { toast } from 'sonner'

import { Button } from '@/components/button'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import type { Group } from '@/entities/group/model/types'

interface GroupHeaderProps {
  group: Group
  showBack?: boolean
}

function truncateMiddle(value: string, start = 8, end = 4) {
  if (value.length <= start + end + 3) return value
  return `${value.slice(0, start)}...${value.slice(-end)}`
}

export function GroupHeader({ group, showBack = true }: GroupHeaderProps) {
  const navigate = useRealmNavigate()

  const copyId = () => {
    void navigator.clipboard
      .writeText(group.id)
      .then(() => toast.success('Group ID copied.'))
      .catch(() => toast.error('Failed to copy group ID.'))
  }

  return (
    <header className="bg-background/95 supports-backdrop-filter:bg-background/60 sticky top-0 z-20 flex h-16 shrink-0 items-center justify-between px-6 backdrop-blur">
      <div className="flex min-w-0 items-center gap-4">
        <div className="bg-primary/10 flex h-10 w-10 items-center justify-center rounded-lg">
          <GroupIcon className="text-primary h-5 w-5" />
        </div>

        <div className="flex min-w-0 flex-col">
          <div className="flex items-center gap-2">
            <h1 className="text-foreground truncate text-lg font-bold tracking-tight">
              {group.name}
            </h1>
          </div>
          <div className="text-muted-foreground flex min-w-0 items-center gap-1 text-xs">
            <span>ID:</span>
            <span className="font-mono" title={group.id}>
              {truncateMiddle(group.id)}
            </span>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="h-6 w-6 shrink-0"
              onClick={copyId}
              aria-label="Copy group ID"
            >
              <Copy className="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>
      </div>

      {showBack ? (
        <div className="flex items-center gap-3">
          <Button variant="outline" onClick={() => navigate('/groups')} size="sm">
            Back
          </Button>
        </div>
      ) : null}
    </header>
  )
}
