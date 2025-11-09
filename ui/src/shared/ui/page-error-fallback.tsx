import { type FC } from 'react'

import { AlertTriangle } from 'lucide-react'

import { Button } from '@/shared/ui/button'

export const PageErrorFallback: FC<{
  error: Error
  resetErrorBoundary: () => void
}> = ({ error, resetErrorBoundary }) => {
  return (
    <div className="bg-background flex h-screen w-full flex-col items-center justify-center gap-4 p-4">
      <AlertTriangle className="text-destructive h-10 w-10" />
      <h2 className="text-lg font-semibold">Something went wrong</h2>
      <p className="text-muted-foreground max-w-lg text-center text-sm">{error?.message}</p>
      <div className="mt-3">
        <Button onClick={resetErrorBoundary}>Try again</Button>
      </div>
    </div>
  )
}
