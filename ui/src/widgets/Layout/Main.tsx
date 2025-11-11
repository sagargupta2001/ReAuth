import type { HTMLAttributes, Ref } from 'react'

import { Search } from '@/features/Search/components/Search.tsx'
import { ThemeSwitch } from '@/features/ThemeSwitch/ThemeSwitch.tsx'
import { ProfileDropdown } from '@/features/auth/ProfileDropdown.tsx'
import { cn } from '@/lib/utils'
import { ConfigDrawer } from '@/widgets/ConfigDrawer/ConfigDrawer.tsx'
import { Header } from '@/widgets/Layout/components/header.tsx'

type MainProps = HTMLAttributes<HTMLElement> & {
  fixed?: boolean
  fluid?: boolean
  ref?: Ref<HTMLElement>
}

export function Main({ fixed, className, fluid, ...props }: MainProps) {
  return (
    <>
      <Header>
        <Search />
        <div className="ms-auto flex items-center gap-4">
          <ThemeSwitch />
          <ConfigDrawer />
          <ProfileDropdown />
        </div>
      </Header>
      <main
        data-layout={fixed ? 'fixed' : 'auto'}
        className={cn(
          'px-4 py-6',

          // If layout is fixed, make the main container flex and grow
          fixed && 'flex grow flex-col overflow-hidden',

          // If layout is not fluid, set the max-width
          !fluid && '@7xl/content:mx-auto @7xl/content:w-full @7xl/content:max-w-7xl',
          className,
        )}
        {...props}
      />
    </>
  )
}
