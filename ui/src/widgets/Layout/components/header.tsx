import { type HTMLAttributes, type ReactNode, type Ref, useEffect, useState } from 'react'

import { cn } from '@/lib/utils.ts'
import { useActivePrimaryNavItem } from '@/widgets/Sidebar/hooks/useActivePrimaryNavItem'
import { RealmSwitcher } from '@/widgets/Layout/components/realm-switcher.tsx'

type HeaderProps = HTMLAttributes<HTMLElement> & {
  fixed?: boolean
  ref?: Ref<HTMLElement>
  leftSlot?: ReactNode
}

export function Header({ className, fixed, children, leftSlot, ...props }: HeaderProps) {
  const [offset, setOffset] = useState(0)
  const activeItem = useActivePrimaryNavItem()

  useEffect(() => {
    const onScroll = () => {
      setOffset(document.body.scrollTop || document.documentElement.scrollTop)
    }

    document.addEventListener('scroll', onScroll, { passive: true })

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
          'relative flex h-full w-full items-center justify-between gap-3 pr-4 sm:pr-6',
          offset > 10 &&
            fixed &&
            'after:bg-background/20 after:absolute after:inset-0 after:-z-10 after:backdrop-blur-lg',
        )}
      >
        <div className="flex min-w-0 items-center gap-1">
          {leftSlot ? (
            leftSlot
          ) : (
            <>
              <div className="shrink-0 overflow-hidden pl-3" style={{ width: '12.5rem' }}>
                <RealmSwitcher />
              </div>
              {activeItem && (
                <>
                  <div className="mx-1 h-5 w-px shrink-0" />
                  <div className="flex min-w-0 items-center gap-1.5">
                    <activeItem.icon className="text-foreground h-5 w-5 shrink-0" />
                    <span className="text-foreground truncate text-lg font-medium">
                      {activeItem.title}
                    </span>
                  </div>
                </>
              )}
            </>
          )}
        </div>
        {/* Right side: rest of header */}
        {children}
      </div>
    </header>
  )
}
