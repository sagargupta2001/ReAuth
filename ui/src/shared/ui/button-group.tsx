import * as React from 'react'
import { cn } from '@/lib/utils'

export type ButtonGroupProps = React.HTMLAttributes<HTMLDivElement>

export function ButtonGroup({ className, ...props }: ButtonGroupProps) {
  return (
    <div
      className={cn(
        'flex w-fit items-center rounded-md',
        'focus-within:ring-2 focus-within:ring-ring focus-within:ring-offset-1',
        '[&>button]:rounded-none [&>button:first-child]:rounded-l-md [&>button:last-child]:rounded-r-md',
        '[&>input]:rounded-none [&>input:first-child]:rounded-l-md [&>input:last-child]:rounded-r-md',
        '[&>input]:focus-visible:ring-0 [&>input]:border-r-0',
        '[&>button]:focus-visible:ring-0',
        '[&>button+button]:-ml-px [&>button+input]:-ml-px [&>input+button]:-ml-px [&>input+input]:-ml-px',
        className
      )}
      {...props}
    />
  )
}
