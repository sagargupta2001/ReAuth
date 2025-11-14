import { Search } from '@/features/Search/components/Search'
import { ThemeSwitch } from '@/features/ThemeSwitch/ThemeSwitch'
import { ProfileDropdown } from '@/features/auth/ProfileDropdown'
import { cn } from '@/lib/utils'
import { Header } from '@/widgets/Layout/components/header.tsx'

export function AppHeader() {
  return (
    <Header
      className={cn(
        'fixed top-0 right-0 left-0 z-50 flex',
        'bg-background/80 supports-[backdrop-filter]:bg-background/60 h-14 border-b backdrop-blur',
      )}
    >
      <div className="flex items-center gap-4">
        <Search />
        <ThemeSwitch />
        <ProfileDropdown />
      </div>
    </Header>
  )
}
