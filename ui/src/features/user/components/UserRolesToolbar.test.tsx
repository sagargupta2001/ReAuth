import { useState } from 'react'

import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'

import { UserRolesToolbar } from './UserRolesToolbar'

function StatefulToolbar({
  onSearchChange,
  onFilterChange,
}: {
  onSearchChange: (value: string) => void
  onFilterChange: (value: 'all' | 'direct' | 'effective' | 'unassigned') => void
}) {
  const [searchValue, setSearchValue] = useState('')

  return (
    <UserRolesToolbar
      searchValue={searchValue}
      onSearchChange={(value) => {
        setSearchValue(value)
        onSearchChange(value)
      }}
      filterValue="all"
      onFilterChange={onFilterChange}
    />
  )
}

describe('UserRolesToolbar', () => {
  it('renders the search input and access filter', () => {
    render(
      <UserRolesToolbar
        searchValue=""
        onSearchChange={vi.fn()}
        filterValue="all"
        onFilterChange={vi.fn()}
      />,
    )

    expect(screen.getByPlaceholderText('Search...')).toHaveClass('pl-10')
    expect(screen.getByRole('button', { name: /filter roles by assignment/i })).toBeInTheDocument()
  })

  it('emits search and single access filter changes', async () => {
    const user = userEvent.setup()
    const onSearchChange = vi.fn()
    const onFilterChange = vi.fn()

    render(
      <StatefulToolbar onSearchChange={onSearchChange} onFilterChange={onFilterChange} />,
    )

    await user.type(screen.getByPlaceholderText('Search...'), 'admin')
    expect(onSearchChange).toHaveBeenLastCalledWith('admin')

    await user.click(screen.getByRole('button', { name: /filter roles by assignment/i }))
    await user.click(screen.getByText('Direct'))

    expect(onFilterChange).toHaveBeenCalledWith('direct')
  })
})
