import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { Collapsible, CollapsibleTrigger, CollapsibleContent } from './collapsible'

describe('Collapsible', () => {
  it('renders correctly and toggles', () => {
    render(
      <Collapsible>
        <CollapsibleTrigger>Toggle</CollapsibleTrigger>
        <CollapsibleContent>
          <div>Collapsible content</div>
        </CollapsibleContent>
      </Collapsible>
    )
    
    expect(screen.queryByText('Collapsible content')).not.toBeInTheDocument()
    
    fireEvent.click(screen.getByText('Toggle'))
    
    expect(screen.getByText('Collapsible content')).toBeInTheDocument()
  })
})
