import { type ReactNode } from 'react'

interface ObservabilityLayoutProps {
  title: string
  description: string
  summary?: ReactNode
  children: ReactNode
}

export function ObservabilityLayout({
  title,
  description,
  summary,
  children,
}: ObservabilityLayoutProps) {
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">{title}</h1>
          <p className="text-muted-foreground text-sm">{description}</p>
        </div>
      </div>

      {summary}

      <div className="flex min-h-0 flex-1 flex-col overflow-hidden">{children}</div>
    </div>
  )
}
