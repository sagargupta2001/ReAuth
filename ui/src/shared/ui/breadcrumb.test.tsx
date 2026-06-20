import { render, screen } from '@testing-library/react'
import { MemoryRouter, Link } from 'react-router-dom'
import { describe, it, expect } from 'vitest'

import {
  Breadcrumb,
  BreadcrumbEllipsis,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from './breadcrumb'

describe('Breadcrumb', () => {
  it('renders a navigable trail with links, separators and a current page', () => {
    render(
      <MemoryRouter>
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem>
              <BreadcrumbLink asChild>
                <Link to="/master">Home</Link>
              </BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
              <BreadcrumbPage>Settings</BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </MemoryRouter>,
    )

    expect(screen.getByRole('navigation', { name: 'breadcrumb' })).toBeInTheDocument()

    const link = screen.getByRole('link', { name: 'Home' })
    expect(link).toHaveAttribute('href', '/master')

    const current = screen.getByText('Settings')
    expect(current).toHaveAttribute('aria-current', 'page')
  })

  it('renders the ellipsis with an accessible label', () => {
    render(
      <BreadcrumbEllipsis />,
    )
    expect(screen.getByText('More')).toBeInTheDocument()
  })
})
