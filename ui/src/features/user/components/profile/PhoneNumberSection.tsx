import { Phone } from 'lucide-react'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/card'
import { useUserPhoneNumbers } from '@/features/user/api/useUserPhoneNumbers.ts'
import { Skeleton } from '@/shared/ui/skeleton.tsx'

import { AddPhoneNumberDialog } from './AddPhoneNumberDialog.tsx'
import { PhoneNumberRow } from './PhoneNumberRow.tsx'

interface PhoneNumberSectionProps {
  userId: string
}

export function PhoneNumberSection({ userId }: PhoneNumberSectionProps) {
  const { data: phoneNumbers, isLoading } = useUserPhoneNumbers(userId)

  if (isLoading)
    return (
      <Card>
        <CardContent className="space-y-3 pt-6">
          <Skeleton className="h-12" />
          <Skeleton className="h-12" />
        </CardContent>
      </Card>
    )

  const list = phoneNumbers ?? []

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>Phone numbers</CardTitle>
        <AddPhoneNumberDialog userId={userId} />
      </CardHeader>
      <CardContent>
        {list.length === 0 ? (
          <div className="flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed py-10 text-center">
            <Phone className="text-muted-foreground h-8 w-8" />
            <p className="text-muted-foreground text-sm">No phone numbers on this account.</p>
          </div>
        ) : (
          <div>
            {list.map((phoneNumber, index) => (
              <PhoneNumberRow
                key={phoneNumber.id}
                phoneNumber={phoneNumber}
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
