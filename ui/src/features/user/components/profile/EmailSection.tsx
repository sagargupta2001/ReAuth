import { Mail } from 'lucide-react'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/card'
import { Skeleton } from '@/shared/ui/skeleton.tsx'

import { useUserEmails } from '@/features/user/api/useUserEmails.ts'
import { AddEmailDialog } from './AddEmailDialog.tsx'
import { EmailRow } from './EmailRow.tsx'

interface EmailSectionProps {
  userId: string
}

export function EmailSection({ userId }: EmailSectionProps) {
  const { data: emails, isLoading } = useUserEmails(userId)

  if (isLoading)
    return (
      <Card>
        <CardContent className="space-y-3 pt-6">
          <Skeleton className="h-12" />
          <Skeleton className="h-12" />
        </CardContent>
      </Card>
    )

  const list = emails ?? []

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>Email addresses</CardTitle>
        <AddEmailDialog userId={userId} />
      </CardHeader>
      <CardContent>
        {list.length === 0 ? (
          <div className="flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed py-10 text-center">
            <Mail className="text-muted-foreground h-8 w-8" />
            <p className="text-muted-foreground text-sm">No email addresses on this account.</p>
          </div>
        ) : (
          <div>
            {list.map((email, index) => (
              <EmailRow
                key={email.id}
                email={email}
                userId={userId}
                isOnly={list.length === 1}
                isFirst={index === 0}
                isLast={index === list.length - 1}
              />
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
