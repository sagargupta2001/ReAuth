import { MoreVertical, Shield, Trash2 } from 'lucide-react'
import { toast } from 'sonner'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/dropdown-menu'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import type { Role } from '@/features/roles/api/useRoles.ts'

interface RoleHeaderProps {
  role: Role
}

export function RoleHeader({ role }: RoleHeaderProps) {
  const navigate = useRealmNavigate()

  const copyId = () => {
    void navigator.clipboard.writeText(role.id)
    toast.success('Role ID copied')
  }

  return (
    <header className="bg-background/95 supports-backdrop-filter:bg-background/60 sticky top-0 z-20 flex h-16 shrink-0 items-center justify-between border-b px-6 backdrop-blur">
      <div className="flex items-center gap-4">
        <div className="bg-primary/10 flex h-10 w-10 items-center justify-center rounded-lg">
          <Shield className="text-primary h-5 w-5" />
        </div>

        <div className="flex flex-col">
          <div className="flex items-center gap-2">
            <h1 className="text-foreground text-lg font-bold tracking-tight">{role.name}</h1>
            {role.client_id ? (
              <Badge variant="outline" className="text-[10px]">
                Client Role
              </Badge>
            ) : (
              <Badge variant="secondary" className="text-[10px]">
                Global Role
              </Badge>
            )}
          </div>
          <div className="text-muted-foreground flex items-center gap-1 text-xs">
            <span>ID:</span>
            <button onClick={copyId} className="hover:text-foreground font-mono hover:underline">
              {role.id.slice(0, 8)}...
            </button>
          </div>
        </div>
      </div>

      <div className="flex items-center gap-3">
        <Button variant="outline" onClick={() => navigate('/roles')} size="sm">
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
              <Trash2 className="mr-2 h-4 w-4" /> Delete Role
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </header>
  )
}
