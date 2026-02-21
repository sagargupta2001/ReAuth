import { AppWindow, Copy, MoreVertical, Shield, ShieldAlert, Trash2 } from 'lucide-react'
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
import type { OidcClient } from '@/entities/oidc/model/types.ts'

interface ClientHeaderProps {
  client: OidcClient
}

export function ClientHeader({ client }: ClientHeaderProps) {
  const navigate = useRealmNavigate()

  // Logic to determine badge status
  const isConfidential = !!client.client_secret
  const isSystem = client.client_id === 'reauth-admin' // Example logic

  const copyId = () => {
    navigator.clipboard.writeText(client.client_id)
    toast.success('Client ID copied to clipboard')
  }

  return (
    <header className="bg-background/95 supports-[backdrop-filter]:bg-background/60 sticky top-0 z-20 flex h-16 shrink-0 items-center justify-between border-b px-6 backdrop-blur">
      <div className="flex items-center gap-4">
        {/* Icon Box */}
        <div className="bg-primary/10 flex h-10 w-10 items-center justify-center rounded-lg">
          <AppWindow className="text-primary h-5 w-5" />
        </div>

        {/* Title & Metadata */}
        <div className="flex flex-col">
          <div className="flex items-center gap-2">
            <h1 className="text-foreground text-lg font-bold tracking-tight">{client.client_id}</h1>

            {/* Status Badges */}
            {isSystem ? (
              <Badge variant="secondary" className="h-5 text-[10px]">
                System
              </Badge>
            ) : isConfidential ? (
              <Badge
                variant="outline"
                className="h-5 gap-1 border-green-200 bg-green-50 text-[10px] text-green-700"
              >
                <Shield className="h-3 w-3" /> Confidential
              </Badge>
            ) : (
              <Badge
                variant="outline"
                className="h-5 gap-1 border-orange-200 bg-orange-50 text-[10px] text-orange-700"
              >
                <ShieldAlert className="h-3 w-3" /> Public
              </Badge>
            )}
          </div>

          {/* ID Copy Snippet */}
          <div className="text-muted-foreground flex items-center gap-1 text-xs">
            <span>ID:</span>
            <button
              onClick={copyId}
              className="hover:text-foreground flex items-center gap-1 font-mono hover:underline"
              title="Copy Client ID"
            >
              {client.id.slice(0, 8)}...
              <Copy className="h-2.5 w-2.5" />
            </button>
          </div>
        </div>
      </div>

      {/* Right Side Actions */}
      <div className="flex items-center gap-3">
        <Button variant="outline" onClick={() => navigate('/clients')} size="sm">
          Back
        </Button>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon">
              <MoreVertical className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={copyId}>Copy Client ID</DropdownMenuItem>
            <DropdownMenuItem className="text-destructive focus:text-destructive">
              <Trash2 className="mr-2 h-4 w-4" /> Delete Client
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </header>
  )
}
