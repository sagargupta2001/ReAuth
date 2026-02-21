import '@testing-library/jest-dom'
import { afterAll, afterEach, beforeAll, vi } from 'vitest'
import { server } from './server'

// Start worker before all tests
beforeAll(() => server.listen({ onUnhandledRequest: 'error' }))

//  Close worker after all tests
afterAll(() => server.close())

// Reset handlers after each test `important for test isolation`
afterEach(() => server.resetHandlers())

// Mock ResizeObserver
class ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}

window.ResizeObserver = ResizeObserver

// Mock scrollIntoView
Element.prototype.scrollIntoView = vi.fn()
