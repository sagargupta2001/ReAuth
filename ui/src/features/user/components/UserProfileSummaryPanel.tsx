import { format } from 'date-fns'
import { CalendarClock, Copy, Fingerprint, Mail, UserRound } from 'lucide-react'
import { toast } from 'sonner'

import { Button } from '@/components/button'
import { useUser } from '@/features/user/api/useUser.ts'
import { Skeleton } from '@/shared/ui/skeleton.tsx'

interface UserProfileSummaryPanelProps {
  userId: string
}

function formatDate(value?: string | null) {
  if (!value) return '-'
  return format(new Date(value), 'MMM d, yyyy, h:mm a')
}

function SummaryRow({
  icon: Icon,
  label,
  value,
  copyable = false,
}: {
  icon: typeof Fingerprint
  label: string
  value: string
  copyable?: boolean
}) {
  const canCopy = copyable && value !== '-'

  const copyValue = () => {
    if (!canCopy) return

    void navigator.clipboard
      .writeText(value)
      .then(() => toast.success(`${label} copied.`))
      .catch(() => toast.error(`Failed to copy ${label.toLowerCase()}.`))
  }

  return (
    // border-b puts bottom borders everywhere; last:border-b-0 removes it from the final row
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

export function UserProfileSummaryPanel({ userId }: UserProfileSummaryPanelProps) {
  const { data: user, isLoading } = useUser(userId)

  if (isLoading) {
    return (
      <div className="space-y-3 xl:sticky xl:top-6">
        <Skeleton className="h-4 w-28" />
        <Skeleton className="h-14 w-full" />
        <Skeleton className="h-14 w-full" />
        <Skeleton className="h-14 w-full" />
      </div>
    )
  }

  return (
    // self-start acts as a secondary shield protecting it from expanding layout constraints
    <div className="flex flex-col self-start xl:sticky xl:top-6">
      <SummaryRow icon={Fingerprint} label="User ID" value={user?.id ?? '-'} copyable />
      <SummaryRow icon={Mail} label="Primary email" value={user?.email ?? '-'} copyable />
      <SummaryRow icon={UserRound} label="Username" value={user?.username ?? '-'} copyable />
      <SummaryRow icon={CalendarClock} label="User since" value={formatDate(user?.created_at)} />
      <SummaryRow
        icon={CalendarClock}
        label="Profile last updated"
        value={formatDate(user?.updated_at)}
      />
    </div>
  )
}
