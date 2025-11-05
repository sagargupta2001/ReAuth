import { Suspense } from 'react'

import { AppRouter } from './AppRouter'

function App() {
  // We only need the manifests for the sidebar
  //const { data, isLoading } = usePlugins()

  return (
    <>
      {/*<Sidebar plugins={data?.manifests || []} isLoading={isLoading} />*/}

      <Suspense fallback={<div>Loading page...</div>}>
        <AppRouter />
      </Suspense>
    </>
  )
}

export default App
