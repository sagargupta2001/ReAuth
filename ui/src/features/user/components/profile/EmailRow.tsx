import { AlertTriangle, CheckCircle2, Mail, MoreHorizontal, Star, XCircle } from 'lucide-react'

import type { UserEmail } from '@/entities/user/model/types.ts'
import { Button } from '@/components/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/dropdown-menu'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/shared/ui/tooltip.tsx'
import { cn } from '@/lib/utils'

import {
  useRemoveUserEmail,
  useSetEmailVerified,
  useSetPrimaryEmail,
} from '@/features/user/api/useUserEmails.ts'

interface EmailRowProps {
  email: UserEmail
  userId: string
  isOnly: boolean
  isFirst: boolean
  isLast: boolean
}

export function EmailRow({ email, userId, isOnly, isFirst, isLast }: EmailRowProps) {
  const setPrimary = useSetPrimaryEmail(userId)
  const setVerified = useSetEmailVerified(userId)
  const remove = useRemoveUserEmail(userId)

  const busy = setPrimary.isPending || setVerified.isPending || remove.isPending

  return (
    <div
      className={cn(
        'bg-primary-foreground flex items-center justify-between border p-3',
        // Fuse rows into one block: only the outer corners are rounded.
        isOnly && 'rounded-2xl',
        !isOnly && isFirst && 'rounded-t-2xl',
        !isOnly && isLast && 'rounded-b-2xl',
        !isOnly && !isFirst && !isLast && 'rounded-none',
        !isLast && 'border-b-0',
      )}
    >
      <div className="flex min-w-0 items-center gap-3">
        <Mail className="text-muted-foreground h-4 w-4 shrink-0" />
        <div className="flex min-w-0 gap-2">
          <span className="truncate text-sm font-medium">{email.email}</span>
          <div className="mt-0.5 flex items-center gap-1.5">
            {email.is_primary && (
              <span
                data-slot="badge"
                className="bg-green-500/5 text-green-600 ring-green-500/20 dark:bg-green-500/15 dark:text-green-400 from-black/[0.02] relative inline-flex shrink-0 items-center rounded-sm bg-linear-to-t px-1 py-0.5 text-xs ring-1 ring-inset"
              >
                <span className="px-0.5">Primary</span>
              </span>
            )}
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  {email.is_verified ? (
                    <CheckCircle2 className="h-4 w-4 cursor-help text-green-600" />
                  ) : (
                    <AlertTriangle className="h-4 w-4 cursor-help text-yellow-500" />
                  )}
                </TooltipTrigger>
                <TooltipContent className="bg-popover text-popover-foreground border">
                  <p className="text-xs">
                    {email.is_verified
                      ? 'This email address is verified.'
                      : 'This email address is not verified.'}
                  </p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          </div>
        </div>
      </div>

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="icon" className="h-8 w-8 shrink-0" disabled={busy}>
            <MoreHorizontal className="h-4 w-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          {!email.is_primary && (
            <DropdownMenuItem onClick={() => setPrimary.mutate(email.id)}>
              <Star className="mr-2 h-4 w-4" />
              Set as primary
            </DropdownMenuItem>
          )}
          <DropdownMenuItem
            onClick={() => setVerified.mutate({ emailId: email.id, is_verified: !email.is_verified })}
          >
            {email.is_verified ? (
              <>
                <XCircle className="mr-2 h-4 w-4" />
                Mark unverified
              </>
            ) : (
              <>
                <CheckCircle2 className="mr-2 h-4 w-4" />
                Mark verified
              </>
            )}
          </DropdownMenuItem>
          {!isOnly && (
            <>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                className="text-destructive focus:text-destructive"
                onClick={() => remove.mutate(email.id)}
                disabled={email.is_primary}
              >
                Remove
              </DropdownMenuItem>
            </>
          )}
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  )
}
