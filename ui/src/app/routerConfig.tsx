import { DashboardPage } from '@/pages/DashboardPage';
import { NotFoundPage } from '@/pages/NotFoundPage';
import type {ComponentType} from 'react';

/**
 * Defines the shape of a static route.
 */
export interface RouteConfig {
    path: string;
    element: ComponentType;
}

/**
 * An array of all static routes in the application.
 * As your app grows, you just add new pages (like a SettingsPage) here.
 */
export const staticRoutes: RouteConfig[] = [
    {
        path: '/',
        element: DashboardPage,
    },
    {
        path: '*',
        element: NotFoundPage,
    },
];