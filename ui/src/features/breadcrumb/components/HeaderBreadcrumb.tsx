import { Fragment } from 'react'

import { ChevronDown } from 'lucide-react'
import { Link } from 'react-router-dom'

import {
  Breadcrumb,
  BreadcrumbEllipsis,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from '@/components/breadcrumb'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/dropdown-menu'
import { DynamicIsland } from '@/components/dynamic-island'
import { cn } from '@/lib/utils'

import { useBreadcrumbTrail } from '../hooks/useBreadcrumbTrail'
import type { BreadcrumbNode } from '../model/types'

/** Beyond this many nodes, the middle collapses into an ellipsis dropdown. */
const MAX_VISIBLE = 4

function NodeLabel({ node }: { node: BreadcrumbNode }) {
  const Icon = node.icon
  return (
    <>
      {Icon ? <Icon className="size-3.5 shrink-0" /> : null}
      <span className="max-w-[12rem] truncate">{node.label}</span>
    </>
  )
}

function NodeContent({ node }: { node: BreadcrumbNode }) {
  // Quick-switch: a node with peers opens a dropdown to jump sideways.
  if (node.siblings && node.siblings.length > 0) {
    return (
      <DropdownMenu>
        <DropdownMenuTrigger
          className={cn(
            'hover:text-foreground data-[state=open]:text-foreground inline-flex items-center gap-1 outline-none transition-colors',
            node.isCurrent && 'text-foreground font-medium',
          )}
        >
          <NodeLabel node={node} />
          <ChevronDown className="size-3 shrink-0 opacity-60" />
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start">
          {node.siblings.map((s) => {
            const SiblingIcon = s.icon
            return (
              <DropdownMenuItem key={s.href} asChild>
                <Link to={s.href} className="gap-2">
                  {SiblingIcon ? <SiblingIcon className="size-4 shrink-0 opacity-70" /> : null}
                  {s.label}
                </Link>
              </DropdownMenuItem>
            )
          })}
        </DropdownMenuContent>
      </DropdownMenu>
    )
  }

  if (node.isCurrent) {
    return (
      <BreadcrumbPage>
        <NodeLabel node={node} />
      </BreadcrumbPage>
    )
  }

  if (node.href) {
    return (
      <BreadcrumbLink asChild>
        <Link to={node.href}>
          <NodeLabel node={node} />
        </Link>
      </BreadcrumbLink>
    )
  }

  // Section without an index page (e.g. Settings) — plain, non-interactive text.
  return (
    <span className="inline-flex items-center gap-1.5">
      <NodeLabel node={node} />
    </span>
  )
}

/**
 * The centered, animated "Dynamic Island" breadcrumb. Derives its trail from the
 * URL, collapses long trails into an ellipsis dropdown, and morphs fluidly on
 * every navigation. Renders nothing on routes with a trivial trail.
 */
export function HeaderBreadcrumb() {
  const trail = useBreadcrumbTrail()
  if (trail.length === 0) return null

  // Collapse the middle when long: [first] … [last two].
  let visible = trail
  let collapsed: BreadcrumbNode[] = []
  if (trail.length > MAX_VISIBLE) {
    visible = [trail[0], ...trail.slice(-2)]
    collapsed = trail.slice(1, trail.length - 2)
  }

  const contentKey = trail
    .map((n) => `${n.id}:${typeof n.label === 'string' ? n.label : ''}`)
    .join('>')

  return (
    <DynamicIsland contentKey={contentKey} ariaLabel="Breadcrumb">
      <Breadcrumb>
        <BreadcrumbList className="flex-nowrap">
          {visible.map((node, i) => {
            const showEllipsisAfter = collapsed.length > 0 && i === 0
            return (
              <Fragment key={node.id}>
                <BreadcrumbItem>
                  <NodeContent node={node} />
                </BreadcrumbItem>

                {showEllipsisAfter && (
                  <>
                    <BreadcrumbSeparator />
                    <BreadcrumbItem>
                      <DropdownMenu>
                        <DropdownMenuTrigger className="outline-none">
                          <BreadcrumbEllipsis className="hover:text-foreground transition-colors" />
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="start">
                          {collapsed.map((c) => (
                            <DropdownMenuItem key={c.id} asChild={!!c.href}>
                              {c.href ? <Link to={c.href}>{c.label}</Link> : <span>{c.label}</span>}
                            </DropdownMenuItem>
                          ))}
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </BreadcrumbItem>
                  </>
                )}

                {i < visible.length - 1 && <BreadcrumbSeparator />}
              </Fragment>
            )
          })}
        </BreadcrumbList>
      </Breadcrumb>
    </DynamicIsland>
  )
}
