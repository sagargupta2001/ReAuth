import { render, screen } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { DataTable } from './data-table'
import { type ColumnDef } from '@tanstack/react-table'

interface TestData {
  id: string
  name: string
}

const columns: ColumnDef<TestData>[] = [
  { accessorKey: 'id', header: 'ID' },
  { accessorKey: 'name', header: 'Name' },
]

const data: TestData[] = [
  { id: '1', name: 'Item 1' },
  { id: '2', name: 'Item 2' },
]

describe('DataTable', () => {
  it('renders correctly with data', () => {
    render(<DataTable columns={columns} data={data} pageCount={1} />)
    
    expect(screen.getByText('Item 1')).toBeInTheDocument()
    expect(screen.getByText('Item 2')).toBeInTheDocument()
    expect(screen.getByText('ID')).toBeInTheDocument()
    expect(screen.getByText('Name')).toBeInTheDocument()
  })

  it('renders no results message when data is empty', () => {
    render(<DataTable columns={columns} data={[]} pageCount={0} />)
    expect(screen.getByText('No results.')).toBeInTheDocument()
  })
})
