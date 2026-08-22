import type { ReactNode } from 'react'

import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/shared/ui/card'
import { cn } from '@/lib/utils'

/**
 * The inset panel every settings card puts its content in.
 *
 * Exported for the rare caller that needs the panel without the card around it;
 * prefer `SectionCard`.
 */
export const SECTION_CARD_PANEL = 'bg-primary-foreground rounded-2xl p-4'

interface SectionCardProps {
  title: ReactNode
  description?: ReactNode
  children: ReactNode
  footer?: ReactNode
  /** Replaces the panel's default `space-y-4` (e.g. `'flex items-center gap-4'`). */
  contentClassName?: string
  className?: string
}

/**
 * Standard settings card: elevated card, header, and an inset rounded panel
 * holding the content.
 *
 * `CardContent` is deliberately `p-1`, so the inset panel is what supplies the
 * content padding. Hand-writing that panel is easy to forget — the setup page
 * shipped without it, which left its fields at a different inset from its title.
 */
export function SectionCard({
  title,
  description,
  children,
  footer,
  contentClassName,
  className,
}: SectionCardProps) {
  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        {description && <CardDescription>{description}</CardDescription>}
      </CardHeader>
      <CardContent>
        <div className={cn(SECTION_CARD_PANEL, contentClassName ?? 'space-y-4')}>
          {children}
        </div>
      </CardContent>
      {footer && (
        <CardFooter className="text-muted-foreground px-6 py-4 text-xs">{footer}</CardFooter>
      )}
    </Card>
  )
}
