import { CacheManager } from '@/features/observability/components/CacheManager'
import { Main } from '@/widgets/Layout/Main'

export function CachePage() {
  return (
    <Main fixed className="flex h-full flex-col gap-6 p-12">
      <div className="space-y-1">
        <h1 className="text-2xl font-semibold tracking-tight">Cache</h1>
        <p className="text-sm text-muted-foreground">
          Monitor cache health and manage namespace flush actions.
        </p>
      </div>
      <CacheManager />
    </Main>
  )
}

export default CachePage
