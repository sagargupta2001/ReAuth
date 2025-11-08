import { Skeleton } from '@/components/skeleton'

interface GlobalLoadingScreenProps {
  message?: string
}

export function GlobalLoader({ message = 'Loading...' }: GlobalLoadingScreenProps) {
  return (
    <div className="bg-background/80 fixed inset-0 z-50 flex flex-col items-center justify-center backdrop-blur-sm">
      <div className="flex flex-col items-center justify-center space-y-8">
        <div className="space-y-2">
          <Skeleton className="bg-muted/50 h-8 w-[250px]" />
          <Skeleton className="bg-muted/50 h-8 w-[200px]" />
        </div>

        <div className="flex flex-col items-center justify-center space-y-4">
          <div className="flex space-x-2">
            <div className="bg-primary h-4 w-4 animate-bounce rounded-full [animation-delay:-0.3s]" />
            <div className="bg-primary h-4 w-4 animate-bounce rounded-full [animation-delay:-0.15s]" />
            <div className="bg-primary h-4 w-4 animate-bounce rounded-full" />
          </div>
          <p className="text-muted-foreground text-sm">{message}</p>
        </div>
      </div>
    </div>
  )
}
