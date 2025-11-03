// src/components/Sidebar.tsx
import { NavLink } from 'react-router-dom';
import type { PluginManifest } from '../types';

interface SidebarProps {
    plugins: PluginManifest[];
}

export function Sidebar({ plugins }: SidebarProps) {
    return (
        <nav className="sidebar">
            <div className="sidebar-header">
                <h3>ReAuth</h3>
            </div>
            <ul className="nav-list">
                <li>
                    <NavLink to="/" end>
                        Home
                    </NavLink>
                </li>
            </ul>

            {plugins.length > 0 && (
                <>
                    <hr />
                    <div className="plugins-header">
                        <h4>Plugins</h4>
                    </div>
                    <ul className="nav-list">
                        {plugins.map((plugin) => (
                            <li key={plugin.id}>
                                <NavLink to={plugin.frontend.route} end>
                                    {plugin.frontend.sidebarLabel}
                                </NavLink>
                            </li>
                        ))}
                    </ul>
                </>
            )}
        </nav>
    );
}