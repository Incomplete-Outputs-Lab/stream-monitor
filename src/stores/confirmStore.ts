import { create } from 'zustand';

interface ConfirmDialog {
  isOpen: boolean;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  onConfirm: () => void;
  onCancel?: () => void;
  type?: 'info' | 'warning' | 'danger';
}

interface ConfirmStore {
  dialog: ConfirmDialog | null;
  showConfirm: (options: Omit<ConfirmDialog, 'isOpen'>) => void;
  hideConfirm: () => void;
}

export const useConfirmStore = create<ConfirmStore>((set) => ({
  dialog: null,
  
  showConfirm: (options) => {
    set({
      dialog: {
        ...options,
        isOpen: true,
      },
    });
  },
  
  hideConfirm: () => {
    set({ dialog: null });
  },
}));

// グローバルに使用できるconfirm関数
export const showConfirm = (options: Omit<ConfirmDialog, 'isOpen'>) => {
  useConfirmStore.getState().showConfirm(options);
};
