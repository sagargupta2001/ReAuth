import { Suspense } from 'react'

import { AppRouter } from './AppRouter'

function App() {
  return (
    <>
      <Suspense fallback={<div>Loading page...</div>}>
        <AppRouter />
      </Suspense>
    </>
  )
}

export default App
