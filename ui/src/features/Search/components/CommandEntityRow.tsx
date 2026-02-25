import type { ReactNode } from 'react'

import { AppWindow, Group, Radio, Shield, User, Workflow } from 'lucide-react'

import { Avatar, AvatarFallback } from '@/components/avatar'
import { CommandItem } from '@/components/command'

interface CommandEntityRowProps {
  value: string
  kind: 'user' | 'client' | 'role' | 'group' | 'flow' | 'webhook'
  primary: string
  secondary?: string
  onSelect?: () => void
  onHighlight?: () => void
  leading?: ReactNode
}

const kindIcons = {
  user: User,
  client: AppWindow,
  role: Shield,
  group: Group,
  flow: Workflow,
  webhook: Radio,
}

export function CommandEntityRow({
  value,
  kind,
  primary,
  secondary,
  onSelect,
  onHighlight,
  leading,
}: CommandEntityRowProps) {
  const Icon = kindIcons[kind]

  return (
    <CommandItem
      value={value}
      onSelect={onSelect}
      onFocus={onHighlight}
      onMouseEnter={onHighlight}
      className="py-2"
    >
      <div className="flex w-full items-center gap-3">
        {leading ||
          (kind === 'user' ? (
            <Avatar className="h-8 w-8">
              <AvatarFallback className="text-xs font-medium">
                {primary.slice(0, 2).toUpperCase()}
              </AvatarFallback>
            </Avatar>
          ) : (
            <div className="bg-muted/60 text-muted-foreground flex h-8 w-8 items-center justify-center rounded-md">
              <Icon className="h-4 w-4" />
            </div>
          ))}
        <div className="flex flex-1 flex-col">
          <span className="text-sm font-medium">{primary}</span>
          {secondary && <span className="text-xs text-muted-foreground">{secondary}</span>}
        </div>
        <Icon className="text-muted-foreground h-4 w-4" />
      </div>
    </CommandItem>
  )
}
