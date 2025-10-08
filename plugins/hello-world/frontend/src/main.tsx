import HelloWorldPage from './HelloWorldPage';

const pluginId = 'hello-world';

// Compute the global name to match host
function getUMDName(id: string) {
    return id
        .split('-')
        .map((w, i) => (i === 0 ? w : w[0].toUpperCase() + w.slice(1)))
        .join('') + 'Plugin';
}

// Attach to window
(window as any)[getUMDName(pluginId)] = HelloWorldPage;

// Also export default for ESM (optional)
export default HelloWorldPage;
