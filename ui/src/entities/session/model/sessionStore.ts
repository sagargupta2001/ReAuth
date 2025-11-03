import { create } from 'zustand';

// Define the shape of your user and session
interface User {
    id: string;
    username: string;
    // ... other properties
}

interface SessionState {
    user: User | null;
    token: string | null;
    setUser: (user: User, token: string) => void;
    clearSession: () => void;
}

// Create the store
export const useSessionStore = create<SessionState>((set) => ({
    user: null,
    token: null,
    setUser: (user, token) => set({ user, token }),
    clearSession: () => set({ user: null, token: null }),
}));