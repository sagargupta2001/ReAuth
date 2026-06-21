import type { ReactNode } from 'react'

import { cn } from '@/lib/utils'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card'

interface RealmSettingsCardProps {
  title: ReactNode
  description?: ReactNode
  children: ReactNode
  id?: string
  className?: string
  contentClassName?: string
  bodyClassName?: string
}

export function RealmSettingsCard({
  title,
  description,
  children,
  id,
  className,
  contentClassName,
  bodyClassName,
}: RealmSettingsCardProps) {
  return (
    <Card id={id} className={className}>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        {description ? <CardDescription>{description}</CardDescription> : null}
      </CardHeader>
      <CardContent className={contentClassName}>
        <div className={cn('bg-primary-foreground rounded-2xl p-4', bodyClassName)}>{children}</div>
      </CardContent>
    </Card>
  )
}
