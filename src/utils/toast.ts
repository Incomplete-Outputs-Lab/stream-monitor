import { useToastStore } from '../stores/toastStore';

// グローバルに利用できるtoast関数
export const toast = {
  success: (message: string, duration?: number) => {
    useToastStore.getState().addToast(message, 'success', duration);
  },
  error: (message: string, duration?: number) => {
    useToastStore.getState().addToast(message, 'error', duration);
  },
  warning: (message: string, duration?: number) => {
    useToastStore.getState().addToast(message, 'warning', duration);
  },
  info: (message: string, duration?: number) => {
    useToastStore.getState().addToast(message, 'info', duration);
  },
};
