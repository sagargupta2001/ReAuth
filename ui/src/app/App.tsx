import { Suspense } from 'react';
import { Sidebar } from '@/widgets/Sidebar/Sidebar';
import { AppRouter } from './AppRouter';
import { usePlugins } from '@/entities/plugin/api/usePlugins';

import './App.css';

function App() {
    // We only need the manifests for the sidebar
    const { data, isLoading } = usePlugins();

    return (
        <div className="app-container">
            <Sidebar plugins={data?.manifests || []} isLoading={isLoading} />

            <main className="main-content">
                <Suspense fallback={<div>Loading page...</div>}>
                    <AppRouter />
                </Suspense>
            </main>
        </div>
    );
}

export default App;