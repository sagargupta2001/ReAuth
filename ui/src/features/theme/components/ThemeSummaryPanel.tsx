import type { ReactNode } from 'react'

import { format } from 'date-fns'
import {
  CalendarClock,
  CalendarPlus,
  Copy,
  Fingerprint,
  GitBranch,
  Lock,
  type LucideIcon,
  Palette,
} from 'lucide-react'
import { toast } from 'sonner'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { useTheme } from '@/features/theme/api/useTheme'
import { Skeleton } from '@/shared/ui/skeleton'

interface ThemeSummaryPanelProps {
  themeId: string
}

function formatDate(value?: string | null) {
  if (!value) return '-'
  const parsed = new Date(value)
  if (Number.isNaN(parsed.getTime())) return '-'
  return format(parsed, 'MMM d, yyyy, h:mm a')
}

function SummaryRow({
  icon: Icon,
  label,
  value,
  copyable = false,
}: {
  icon: LucideIcon
  label: string
  value: ReactNode
  copyable?: boolean
}) {
  const canCopy = copyable && typeof value === 'string' && value !== '-'

  const copyValue = () => {
    if (!canCopy) return

    void navigator.clipboard
      .writeText(value)
      .then(() => toast.success(`${label} copied.`))
      .catch(() => toast.error(`Failed to copy ${label.toLowerCase()}.`))
  }

  return (
    <div className="border-border/60 border-b py-3 last:border-b-0">
      <div className="text-muted-foreground mb-1 flex items-center gap-2 text-xs font-medium">
        <Icon className="h-3.5 w-3.5" />
        {label}
      </div>
      <div className="flex min-w-0 items-center justify-between gap-2">
        <span className="truncate text-sm font-medium">{value}</span>
        {copyable ? (
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="h-7 w-7 shrink-0"
            disabled={!canCopy}
            onClick={copyValue}
            aria-label={`Copy ${label}`}
          >
            <Copy className="h-3.5 w-3.5" />
          </Button>
        ) : null}
      </div>
    </div>
  )
}

export function ThemeSummaryPanel({ themeId }: ThemeSummaryPanelProps) {
  const { data, isLoading } = useTheme(themeId)

  if (isLoading || !data) {
    return (
      <div className="flex flex-col gap-3 self-start xl:sticky xl:top-6">
        <Skeleton className="h-12 w-full" />
        <Skeleton className="h-12 w-full" />
        <Skeleton className="h-12 w-full" />
        <Skeleton className="h-12 w-full" />
      </div>
    )
  }

  const theme = data.theme
  const activeVersion = data.active_version_number

  return (
    // self-start shields it from expanding layout constraints
    <div className="flex flex-col self-start xl:sticky xl:top-6">
      <SummaryRow icon={Fingerprint} label="Theme ID" value={theme.id} copyable />
      <SummaryRow
        icon={theme.is_system ? Lock : Palette}
        label="Origin"
        value={
          <Badge variant={theme.is_system ? 'secondary' : 'outline'} className="h-5 text-[10px]">
            {theme.is_system ? 'Default' : 'Custom'}
          </Badge>
        }
      />
      <SummaryRow
        icon={GitBranch}
        label="Active version"
        value={
          typeof activeVersion === 'number' ? (
            <Badge variant="secondary" className="h-5 text-[10px]">
              v{activeVersion}
            </Badge>
          ) : (
            <Badge variant="outline" className="h-5 text-[10px]">
              Not active
            </Badge>
          )
        }
      />
      <SummaryRow icon={CalendarPlus} label="Created" value={formatDate(theme.created_at)} />
      <SummaryRow icon={CalendarClock} label="Last updated" value={formatDate(theme.updated_at)} />
    </div>
  )
}
