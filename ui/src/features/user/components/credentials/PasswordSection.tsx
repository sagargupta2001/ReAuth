import { useState } from 'react'

import { LockIcon, MoreHorizontal } from 'lucide-react'

import { Button } from '@/components/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/card'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/dropdown-menu'
import type { UserCredentials } from '@/features/user/api/useUserCredentials'
import { ChangePasswordDialog } from '@/features/user/components/credentials/ChangePasswordDialog'

interface PasswordSectionProps {
  userId: string
  password?: UserCredentials['password']
}

export function PasswordSection({ userId }: PasswordSectionProps) {
  const [changePasswordOpen, setChangePasswordOpen] = useState(false)

  return (
    <Card>
      <CardHeader>
        <CardTitle>Password</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="bg-primary-foreground flex items-center justify-between rounded-2xl border p-3">
          <div className="flex min-w-0 items-center gap-3">
            <LockIcon className="text-muted-foreground h-4 w-4 shrink-0" />
            <span className="text-xs">●●●●●●●●●●</span>
          </div>

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="icon" className="h-8 w-8 shrink-0">
                <MoreHorizontal className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={() => setChangePasswordOpen(true)}>
                Change password
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </CardContent>

      <ChangePasswordDialog
        userId={userId}
        open={changePasswordOpen}
        onOpenChange={setChangePasswordOpen}
      />
    </Card>
  )
}
