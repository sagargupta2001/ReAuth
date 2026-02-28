import { Loader2 } from 'lucide-react'

import type { AuthScreenProps } from '@/entities/auth/model/screenTypes'

export function AwaitingActionScreen({ context }: AuthScreenProps) {
  const message =
    (typeof context.message === 'string' && context.message) ||
    (typeof context.description === 'string' && context.description) ||
    'Waiting for verification...'

  return (
    <div className="flex flex-col items-center gap-3 text-center">
      <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      <div className="text-sm font-medium text-foreground">{message}</div>
      <p className="text-xs text-muted-foreground">
        You can keep this page open or come back after completing the step.
      </p>
    </div>
  )
}
