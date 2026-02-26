import { AppWindow, Group, Radio, Settings, Shield, Workflow, Zap } from 'lucide-react'
import { useNavigate } from 'react-router-dom'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { Avatar, AvatarFallback } from '@/components/avatar'

import type { OmniInspectorItem } from '@/features/Search/model/omniTypes'
import { useSearch } from '@/features/Search/model/searchContext'

interface PaletteInspectorProps {
  item: OmniInspectorItem | null
  descriptionId?: string
  onWebhookAction?: (action: 'enable' | 'disable' | 'roll' | 'delete', id: string, label: string) => void
  webhookActionPending?: string | null
}

export function PaletteInspector({
  item,
  descriptionId,
  onWebhookAction,
  webhookActionPending,
}: PaletteInspectorProps) {
  const navigate = useNavigate()
  const { setOpen } = useSearch()

  const handleOpen = () => {
    if (!item?.href) return
    setOpen(false)
    navigate(item.href)
  }

  if (!item) {
    return (
      <div className="flex h-full flex-col items-center justify-center px-6 text-center text-sm text-muted-foreground">
        {descriptionId && (
          <div id={descriptionId} className="sr-only">
            No selection
          </div>
        )}
        <Zap className="mb-3 h-5 w-5" />
        Select a result to preview details.
      </div>
    )
  }

  if (item.kind === 'user') {
    return (
      <div className="flex h-full flex-col gap-6 p-6">
        {descriptionId && (
          <div id={descriptionId} className="sr-only">
            Selected {item.label}
          </div>
        )}
        <div className="flex items-center gap-4">
          <Avatar className="h-12 w-12">
            <AvatarFallback className="text-sm font-semibold">
              {item.label.slice(0, 2).toUpperCase()}
            </AvatarFallback>
          </Avatar>
          <div>
            <div className="text-base font-semibold">{item.label}</div>
            <div className="text-xs text-muted-foreground">User</div>
          </div>
        </div>
        <div className="space-y-3 text-sm">
          <div className="flex items-center gap-2">
            <Badge variant="secondary">Status: Active</Badge>
          </div>
          {item.subtitle && (
            <div className="text-xs text-muted-foreground">ID: {item.subtitle}</div>
          )}
          <div className="text-xs text-muted-foreground">Last login: Not available</div>
        </div>
        <div className="flex flex-col gap-2">
          <Button variant="secondary" size="sm" onClick={handleOpen}>
            Open Profile
          </Button>
          <Button variant="outline" size="sm" disabled>
            Reset Password
          </Button>
          <Button variant="outline" size="sm" disabled>
            View Audit Logs
          </Button>
        </div>
      </div>
    )
  }

  if (item.kind === 'setting') {
    return (
      <div className="flex h-full flex-col gap-4 p-6">
        {descriptionId && (
          <div id={descriptionId} className="sr-only">
            Selected {item.label}
          </div>
        )}
        <div className="bg-muted/60 text-muted-foreground flex h-12 w-12 items-center justify-center rounded-xl">
          <Settings className="h-5 w-5" />
        </div>
        <div>
          <div className="text-base font-semibold">{item.label}</div>
          {item.description && (
            <p className="mt-2 text-sm text-muted-foreground">{item.description}</p>
          )}
        </div>
        {item.breadcrumb && (
          <div className="text-xs text-muted-foreground">{item.breadcrumb}</div>
        )}
        {item.href && (
          <div className="mt-2">
            <Button variant="secondary" size="sm" onClick={handleOpen}>
              Open Setting
            </Button>
          </div>
        )}
      </div>
    )
  }

  if (item.kind === 'client') {
    return (
      <div className="flex h-full flex-col gap-4 p-6">
        {descriptionId && (
          <div id={descriptionId} className="sr-only">
            Selected {item.label}
          </div>
        )}
        <div className="bg-muted/60 text-muted-foreground flex h-12 w-12 items-center justify-center rounded-xl">
          <AppWindow className="h-5 w-5" />
        </div>
        <div>
          <div className="text-base font-semibold">{item.label}</div>
          {item.subtitle && <p className="mt-2 text-sm text-muted-foreground">{item.subtitle}</p>}
        </div>
        <div className="text-xs text-muted-foreground">Client</div>
        <div className="mt-2">
          <Button variant="secondary" size="sm" onClick={handleOpen}>
            Open Client
          </Button>
        </div>
      </div>
    )
  }

  if (item.kind === 'role') {
    return (
      <div className="flex h-full flex-col gap-4 p-6">
        {descriptionId && (
          <div id={descriptionId} className="sr-only">
            Selected {item.label}
          </div>
        )}
        <div className="bg-muted/60 text-muted-foreground flex h-12 w-12 items-center justify-center rounded-xl">
          <Shield className="h-5 w-5" />
        </div>
        <div>
          <div className="text-base font-semibold">{item.label}</div>
          {item.subtitle && <p className="mt-2 text-sm text-muted-foreground">{item.subtitle}</p>}
        </div>
        <div className="text-xs text-muted-foreground">Role</div>
        <div className="mt-2">
          <Button variant="secondary" size="sm" onClick={handleOpen}>
            Open Role
          </Button>
        </div>
      </div>
    )
  }

  if (item.kind === 'action') {
    return (
      <div className="flex h-full flex-col gap-4 p-6">
        {descriptionId && (
          <div id={descriptionId} className="sr-only">
            Selected {item.label}
          </div>
        )}
        <div className="bg-muted/60 text-muted-foreground flex h-12 w-12 items-center justify-center rounded-xl">
          <Zap className="h-5 w-5" />
        </div>
        <div className="text-base font-semibold">{item.label}</div>
        {item.description && <p className="text-sm text-muted-foreground">{item.description}</p>}
        {item.href && (
          <div>
            <Button variant="secondary" size="sm" onClick={handleOpen}>
              Open
            </Button>
          </div>
        )}
      </div>
    )
  }

  if (item.kind === 'group') {
    return (
      <div className="flex h-full flex-col gap-4 p-6">
        {descriptionId && (
          <div id={descriptionId} className="sr-only">
            Selected {item.label}
          </div>
        )}
        <div className="bg-muted/60 text-muted-foreground flex h-12 w-12 items-center justify-center rounded-xl">
          <Group className="h-5 w-5" />
        </div>
        <div>
          <div className="text-base font-semibold">{item.label}</div>
          {item.subtitle && <p className="mt-2 text-sm text-muted-foreground">{item.subtitle}</p>}
        </div>
        <div className="text-xs text-muted-foreground">Group</div>
        {item.href && (
          <div className="mt-2">
            <Button variant="secondary" size="sm" onClick={handleOpen}>
              Open Group
            </Button>
          </div>
        )}
      </div>
    )
  }

  if (item.kind === 'flow') {
    return (
      <div className="flex h-full flex-col gap-4 p-6">
        {descriptionId && (
          <div id={descriptionId} className="sr-only">
            Selected {item.label}
          </div>
        )}
        <div className="bg-muted/60 text-muted-foreground flex h-12 w-12 items-center justify-center rounded-xl">
          <Workflow className="h-5 w-5" />
        </div>
        <div>
          <div className="text-base font-semibold">{item.label}</div>
          {item.description && (
            <p className="mt-2 text-sm text-muted-foreground">{item.description}</p>
          )}
        </div>
        {item.subtitle && <div className="text-xs text-muted-foreground">{item.subtitle}</div>}
        {item.href && (
          <div className="mt-2">
            <Button variant="secondary" size="sm" onClick={handleOpen}>
              Open Flow
            </Button>
          </div>
        )}
      </div>
    )
  }

  if (item.kind === 'webhook') {
    const isActive = item.status === 'active'
    const toggleAction = isActive ? 'disable' : 'enable'
    const toggleLabel = isActive ? 'Disable' : 'Enable'
    const isPending = (action: string) => webhookActionPending === action
    return (
      <div className="flex h-full flex-col gap-4 p-6">
        {descriptionId && (
          <div id={descriptionId} className="sr-only">
            Selected {item.label}
          </div>
        )}
        <div className="bg-muted/60 text-muted-foreground flex h-12 w-12 items-center justify-center rounded-xl">
          <Radio className="h-5 w-5" />
        </div>
        <div>
          <div className="text-base font-semibold">{item.label}</div>
          {item.subtitle && <p className="mt-2 text-sm text-muted-foreground">{item.subtitle}</p>}
        </div>
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <span>Webhook Endpoint</span>
          {item.description && (
            <Badge variant={item.description === 'active' ? 'success' : 'destructive'}>
              {item.description}
            </Badge>
          )}
        </div>
        <div className="mt-2 flex flex-col gap-2">
          {item.href && (
            <Button variant="secondary" size="sm" onClick={handleOpen}>
              Open Webhook
            </Button>
          )}
          <div className="grid gap-2 md:grid-cols-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => onWebhookAction?.(toggleAction, item.id, item.label)}
              disabled={!onWebhookAction || isPending(toggleAction)}
            >
              {toggleLabel}
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => onWebhookAction?.('roll', item.id, item.label)}
              disabled={!onWebhookAction || isPending('roll')}
            >
              Roll Secret
            </Button>
          </div>
          <Button
            variant="destructive"
            size="sm"
            onClick={() => onWebhookAction?.('delete', item.id, item.label)}
            disabled={!onWebhookAction || isPending('delete')}
          >
            Delete Webhook
          </Button>
        </div>
      </div>
    )
  }

  return null
}
