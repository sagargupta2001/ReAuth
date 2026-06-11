import { AlertTriangle, CheckCircle2, MoreHorizontal, Phone, Star, XCircle } from 'lucide-react'

import type { UserPhoneNumber } from '@/entities/user/model/types.ts'
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
  useRemoveUserPhoneNumber,
  useSetPhoneNumberVerified,
  useSetPrimaryPhoneNumber,
} from '@/features/user/api/useUserPhoneNumbers.ts'

interface PhoneNumberRowProps {
  phoneNumber: UserPhoneNumber
  userId: string
  isOnly: boolean
  isFirst: boolean
  isLast: boolean
}

export function PhoneNumberRow({
  phoneNumber,
  userId,
  isOnly,
  isFirst,
  isLast,
}: PhoneNumberRowProps) {
  const setPrimary = useSetPrimaryPhoneNumber(userId)
  const setVerified = useSetPhoneNumberVerified(userId)
  const remove = useRemoveUserPhoneNumber(userId)

  const busy = setPrimary.isPending || setVerified.isPending || remove.isPending

  return (
    <div
      className={cn(
        'bg-primary-foreground flex items-center justify-between border p-3',
        isOnly && 'rounded-2xl',
        !isOnly && isFirst && 'rounded-t-2xl',
        !isOnly && isLast && 'rounded-b-2xl',
        !isOnly && !isFirst && !isLast && 'rounded-none',
        !isLast && 'border-b-0',
      )}
    >
      <div className="flex min-w-0 items-center gap-3">
        <Phone className="text-muted-foreground h-4 w-4 shrink-0" />
        <div className="flex min-w-0 gap-2">
          <span className="truncate text-sm font-medium">{phoneNumber.phone_number}</span>
          <div className="mt-0.5 flex items-center gap-1.5">
            {phoneNumber.is_primary && (
              <span
                data-slot="badge"
                className="from-black/[0.02] relative inline-flex shrink-0 items-center rounded-sm bg-green-500/5 bg-linear-to-t px-1 py-0.5 text-xs text-green-600 ring-1 ring-green-500/20 ring-inset dark:bg-green-500/15 dark:text-green-400"
              >
                <span className="px-0.5">Primary</span>
              </span>
            )}
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  {phoneNumber.is_verified ? (
                    <CheckCircle2 className="h-4 w-4 cursor-help text-green-600" />
                  ) : (
                    <AlertTriangle className="h-4 w-4 cursor-help text-yellow-500" />
                  )}
                </TooltipTrigger>
                <TooltipContent className="bg-popover text-popover-foreground border">
                  <p className="text-xs">
                    {phoneNumber.is_verified
                      ? 'This phone number is verified.'
                      : 'This phone number is not verified.'}
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
          {!phoneNumber.is_primary && (
            <DropdownMenuItem onClick={() => setPrimary.mutate(phoneNumber.id)}>
              <Star className="mr-2 h-4 w-4" />
              Set as primary
            </DropdownMenuItem>
          )}
          <DropdownMenuItem
            onClick={() =>
              setVerified.mutate({
                phoneNumberId: phoneNumber.id,
                is_verified: !phoneNumber.is_verified,
              })
            }
          >
            {phoneNumber.is_verified ? (
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
                onClick={() => remove.mutate(phoneNumber.id)}
                disabled={phoneNumber.is_primary}
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
