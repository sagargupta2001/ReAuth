import type { HTMLAttributes } from 'react'

import { Menu } from 'lucide-react'
import { Link, useLocation } from 'react-router-dom'

import { cn } from '@/lib/utils.ts'
import { Button } from '@/shared/ui/button.tsx'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/shared/ui/dropdown-menu.tsx'

type TopNavProps = HTMLAttributes<HTMLElement> & {
  links: {
    title: string
    href: string
    isActive: boolean
    disabled?: boolean
  }[]
}

export function TopNav({ className, links, ...props }: TopNavProps) {
  const location = useLocation()

  return (
    <>
      {/* Mobile Nav */}
      <div className="lg:hidden">
        <DropdownMenu modal={false}>
          <DropdownMenuTrigger asChild>
            <Button size="icon" variant="outline" className="md:size-7">
              <Menu />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent side="bottom" align="start">
            {links.map(({ title, href, disabled }) => {
              const isActive = location.pathname === href
              return (
                <DropdownMenuItem key={`${title}-${href}`} asChild>
                  {disabled ? (
                    <span className={cn('cursor-not-allowed opacity-50', 'text-muted-foreground')}>
                      {title}
                    </span>
                  ) : (
                    <Link to={href} className={cn(!isActive && 'text-muted-foreground')}>
                      {title}
                    </Link>
                  )}
                </DropdownMenuItem>
              )
            })}
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      {/* Desktop Nav */}
      <nav
        className={cn('hidden items-center space-x-4 lg:flex lg:space-x-4 xl:space-x-6', className)}
        {...props}
      >
        {links.map(({ title, href, disabled }) => {
          const isActive = location.pathname === href
          return disabled ? (
            <span
              key={`${title}-${href}`}
              className={cn(
                'cursor-not-allowed text-sm font-medium opacity-50 transition-colors',
                'text-muted-foreground',
              )}
            >
              {title}
            </span>
          ) : (
            <Link
              key={`${title}-${href}`}
              to={href}
              className={cn(
                'text-sm font-medium transition-colors hover:text-primary',
                !isActive && 'text-muted-foreground',
              )}
            >
              {title}
            </Link>
          )
        })}
      </nav>
    </>
  )
}
