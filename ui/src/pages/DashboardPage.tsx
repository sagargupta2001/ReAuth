import { Button } from '@/shared/ui/button';

export function DashboardPage() {
    return (
        <div>
            <h1 className="text-3xl font-bold">Welcome to ReAuth Core</h1>
            <p className="text-muted-foreground">Select a plugin from the sidebar.</p>
            <Button className="mt-4">Shadcn Button</Button>
        </div>
    );
}