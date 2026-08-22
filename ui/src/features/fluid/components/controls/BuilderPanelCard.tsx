import type { ReactNode } from 'react'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card'
import { cn } from '@/lib/utils'

interface BuilderPanelCardProps {
  title: string
  description?: string
  children: ReactNode
  /** Replaces the inset panel's default spacing (e.g. `'grid gap-3'`). */
  contentClassName?: string
  className?: string
}

/**
 * Card shell for the builder's side panels.
 *
 * Mirrors the settings-page card pattern — elevated card, header, and an inset
 * rounded panel around the content — so the theme settings and inspector
 * sidebars read the same as the rest of the app.
 */
export function BuilderPanelCard({
  title,
  description,
  children,
  contentClassName,
  className,
}: BuilderPanelCardProps) {
  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        {description && <CardDescription>{description}</CardDescription>}
      </CardHeader>
      <CardContent>
        <div
          className={cn(
            'bg-primary-foreground rounded-2xl p-4',
            contentClassName ?? 'space-y-4',
          )}
        >
          {children}
        </div>
      </CardContent>
    </Card>
  )
}
