import '@testing-library/jest-dom'
import { afterAll, afterEach, beforeAll } from 'vitest'
import { server } from './server'

// Start worker before all tests
beforeAll(() => server.listen({ onUnhandledRequest: 'error' }))

//  Close worker after all tests
afterAll(() => server.close())

// Reset handlers after each test `important for test isolation`
afterEach(() => server.resetHandlers())
