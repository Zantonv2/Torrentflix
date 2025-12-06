import { writable } from "svelte/store";

export interface Notification {
    id: string;
    type: "success" | "error" | "warning" | "info";
    message: string;
    timestamp: Date;
    read: boolean;
}

interface NotificationState {
    notifications: Notification[];
    unreadCount: number;
}

const initialState: NotificationState = {
    notifications: [],
    unreadCount: 0,
};

function createNotificationStore() {
    const { subscribe, set, update } = writable<NotificationState>(initialState);

    return {
        subscribe,
        addNotification: (
            message: string,
            type: "success" | "error" | "warning" | "info" = "info",
        ) => {
            const notification: Notification = {
                id: `notif-${Date.now()}-${Math.random()}`,
                type,
                message,
                timestamp: new Date(),
                read: false,
            };

            update((state) => ({
                ...state,
                notifications: [notification, ...state.notifications].slice(0, 50), // Keep max 50
                unreadCount: state.unreadCount + 1,
            }));

            return notification.id;
        },

        markAsRead: (id: string) => {
            update((state) => {
                const notification = state.notifications.find((n) => n.id === id);
                if (notification && !notification.read) {
                    notification.read = true;
                    return {
                        ...state,
                        unreadCount: Math.max(0, state.unreadCount - 1),
                    };
                }
                return state;
            });
        },

        markAllAsRead: () => {
            update((state) => ({
                ...state,
                notifications: state.notifications.map((n) => ({ ...n, read: true })),
                unreadCount: 0,
            }));
        },

        clearAll: () => {
            update((state) => ({
                ...state,
                notifications: [],
                unreadCount: 0,
            }));
        },

        removeNotification: (id: string) => {
            update((state) => {
                const notification = state.notifications.find((n) => n.id === id);
                return {
                    ...state,
                    notifications: state.notifications.filter((n) => n.id !== id),
                    unreadCount:
                        notification && !notification.read
                            ? Math.max(0, state.unreadCount - 1)
                            : state.unreadCount,
                };
            });
        },

        reset: () => set(initialState),
    };
}

export const notificationStore = createNotificationStore();
