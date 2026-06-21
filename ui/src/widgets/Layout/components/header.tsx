import { type HTMLAttributes, type ReactNode, type Ref, useEffect, useState } from 'react'

import { Slash } from '@/assets/header/slash.tsx'
import { cn } from '@/lib/utils.ts'
import { RealmSwitcher } from '@/widgets/Layout/components/realm-switcher.tsx'

type HeaderProps = HTMLAttributes<HTMLElement> & {
  fixed?: boolean
  ref?: Ref<HTMLElement>
  leftSlot?: ReactNode
  centerSlot?: ReactNode
}

export function Header({
  className,
  fixed,
  children,
  leftSlot,
  centerSlot,
  ...props
}: HeaderProps) {
  const [offset, setOffset] = useState(0)

  useEffect(() => {
    const onScroll = () => {
      setOffset(document.body.scrollTop || document.documentElement.scrollTop)
    }

    // Add scroll listener to the body
    document.addEventListener('scroll', onScroll, { passive: true })

    // Clean up the event listener on unmount
    return () => document.removeEventListener('scroll', onScroll)
  }, [])

  return (
    <header
      className={cn(
        'z-50 h-16',
        fixed && 'header-fixed peer/header sticky top-0 right-0 left-0 w-full',
        offset > 10 && fixed ? 'shadow' : 'shadow-none',
        className,
      )}
      {...props}
    >
      <div
        className={cn(
          'relative flex h-full w-full items-center justify-between gap-3 px-4 sm:px-6',
          offset > 10 &&
            fixed &&
            'after:bg-background/20 after:absolute after:inset-0 after:-z-10 after:backdrop-blur-lg',
        )}
      >
        {/* If the Layout provides a slot (like SidebarTrigger), use it. */}
        {/* Otherwise, show the default Logo + RealmSwitcher. */}
        <div className="flex items-center gap-2">
          {leftSlot ? (
            leftSlot
          ) : (
            <>
              <img rel="icon" src="/reauth.svg" alt="logo" className="h-7 w-7" />
              <Slash className="inline-block h-5 w-5 shrink-0 leading-none" />
              <RealmSwitcher />
            </>
          )}
        </div>

        {/* Center: dynamic-island breadcrumb. Absolutely centered so it stays
            put regardless of the left/right slot widths. Hidden on small screens
            where horizontal room is scarce. */}
        {centerSlot && (
          <div className="pointer-events-none absolute top-1/2 left-1/2 hidden -translate-x-1/2 -translate-y-1/2 md:block">
            <div className="pointer-events-auto">{centerSlot}</div>
          </div>
        )}

        {/* Right side: rest of header */}
        {children}
      </div>
    </header>
  )
}
