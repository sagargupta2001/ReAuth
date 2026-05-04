'use client'

import * as React from 'react'
import type { JSX } from 'react'

import { ChevronDownIcon, ChevronLeftIcon, ChevronRightIcon } from '@radix-ui/react-icons'
import { DayPicker } from 'react-day-picker'

import { cn } from '@/lib/utils'

import { buttonVariants } from './button'

export type CalendarProps = React.ComponentProps<typeof DayPicker>

function Calendar({
  className,
  classNames,
  showOutsideDays = true,
  ...props
}: CalendarProps): JSX.Element {
  return (
    <DayPicker
      showOutsideDays={showOutsideDays}
      className={cn('p-3', className)}
      classNames={{
        months: 'flex flex-col gap-4 sm:flex-row sm:gap-6',
        month: 'space-y-4',
        month_caption: 'relative flex h-9 items-center justify-center px-8 pt-1',
        caption_label: 'text-sm font-medium',
        nav: 'absolute inset-x-0 top-1 flex items-center justify-between px-1',
        button_previous: cn(
          buttonVariants({ variant: 'outline' }),
          'h-7 w-7 bg-transparent p-0 opacity-70 hover:opacity-100',
        ),
        button_next: cn(
          buttonVariants({ variant: 'outline' }),
          'h-7 w-7 bg-transparent p-0 opacity-70 hover:opacity-100',
        ),
        month_grid: 'w-full border-collapse',
        weekdays: 'flex',
        weekday: 'text-muted-foreground w-9 rounded-md font-normal text-[0.8rem]',
        week: 'mt-1 flex w-full',
        day: 'h-9 w-9 p-0 text-center text-sm',
        day_button: cn(
          buttonVariants({ variant: 'ghost' }),
          'h-8 w-8 p-0 font-normal aria-selected:opacity-100',
        ),
        selected:
          'bg-indigo-500 text-white hover:bg-indigo-500 hover:text-white focus:bg-indigo-500 focus:text-white',
        today: 'bg-accent text-accent-foreground',
        outside: 'text-muted-foreground opacity-40',
        disabled: 'text-muted-foreground opacity-40',
        range_start: 'bg-accent rounded-l-md',
        range_middle: 'bg-indigo-100 text-indigo-800 rounded-none',
        range_end: 'bg-accent rounded-r-md',
        hidden: 'invisible',
        ...classNames,
      }}
      components={{
        Chevron: ({ orientation, className: iconClassName, ...iconProps }) => {
          if (orientation === 'left')
            return <ChevronLeftIcon className={cn('h-4 w-4', iconClassName)} {...iconProps} />
          if (orientation === 'right')
            return <ChevronRightIcon className={cn('h-4 w-4', iconClassName)} {...iconProps} />
          return <ChevronDownIcon className={cn('h-4 w-4', iconClassName)} {...iconProps} />
        },
      }}
      {...props}
    />
  )
}
Calendar.displayName = 'Calendar'

export { Calendar }
