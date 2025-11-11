import { BookOpen, Brain, ChevronRight, Cpu, Package, Plug, Settings } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'

import { Badge, type BadgeProps } from '@/components/badge'

interface Props {
  target: string
}

interface TargetVisuals {
  icon: LucideIcon
  label: string
  badgeVariant: BadgeProps['variant']
  path: string
}

/**
 * Parses the target string and returns its visual components.
 */
function getTargetVisuals(target: string): TargetVisuals {
  if (target.startsWith('plugin:')) {
    const parts = target.split(':')
    return {
      icon: Package,
      label: parts.length > 1 ? parts[1] : 'plugin',
      badgeVariant: 'warning',
      path: parts.slice(2).join('::'), // e.g., "sdk::runner"
    }
  }

  if (target.startsWith('manager::')) {
    return {
      icon: Settings,
      label: 'manager',
      badgeVariant: 'purple',
      path: target.split('::').slice(1).join('::'),
    }
  }

  if (target.startsWith('core::adapters::')) {
    return {
      icon: Plug,
      label: 'adapter',
      badgeVariant: 'outline',
      path: target.split('::').slice(2).join('::'),
    }
  }

  if (target.startsWith('core::application::')) {
    return {
      icon: Brain,
      label: 'application',
      badgeVariant: 'default', // You can customize this
      path: target.split('::').slice(2).join('::'),
    }
  }

  if (target.startsWith('core::domain::')) {
    return {
      icon: BookOpen,
      label: 'domain',
      badgeVariant: 'secondary', // You can customize this
      path: target.split('::').slice(2).join('::'),
    }
  }

  // Fallback for any other core log
  return {
    icon: Cpu,
    label: 'core',
    badgeVariant: 'cool',
    path: target.replace(/^core::/, ''), // Clean up the path
  }
}

/**
 * A small component to render a single part of the log target path.
 */
function TargetPart({ name, isLast = false }: { name: string; isLast?: boolean }) {
  return (
    <div className="flex items-center gap-1.5">
      <span className={isLast ? 'text-foreground font-medium' : 'text-muted-foreground'}>
        {name}
      </span>
      {!isLast && <ChevronRight className="text-muted-foreground h-3.5 w-3.5" />}
    </div>
  )
}

/**
 * Renders a complex log target string as a set of human-readable badges and segments.
 */
export function LogTarget({ target }: Props) {
  const { icon: Icon, label, badgeVariant, path } = getTargetVisuals(target)
  const pathParts = path.split('::').filter((p) => p.length > 0)

  return (
    <div className="flex items-center gap-1.5 font-mono text-xs">
      {/* The Main Badge (e.g., "plugin", "core", "adapter") */}
      <Badge variant={badgeVariant} className="gap-1">
        <Icon className="h-3 w-3" />
        <span>{label}</span>
      </Badge>

      {/* The rest of the path */}
      {pathParts.length > 0 && <ChevronRight className="text-muted-foreground h-3.5 w-3.5" />}
      {pathParts.map((part, i) => (
        <TargetPart key={i} name={part} isLast={i === pathParts.length - 1} />
      ))}
    </div>
  )
}
