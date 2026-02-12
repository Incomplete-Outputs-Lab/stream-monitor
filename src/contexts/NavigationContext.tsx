import { createContext, useContext, ReactNode } from 'react';

type Tab = "dashboard" | "channels" | "statistics" | "timeline" | "export" | "logs" | "settings" | "multiview" | "sqlviewer";

interface NavigationContextType {
  activeTab: Tab;
  setActiveTab: (tab: Tab) => void;
}

const NavigationContext = createContext<NavigationContextType | undefined>(undefined);

export function NavigationProvider({ children, activeTab, setActiveTab }: {
  children: ReactNode;
  activeTab: Tab;
  setActiveTab: (tab: Tab) => void;
}) {
  return (
    <NavigationContext.Provider value={{ activeTab, setActiveTab }}>
      {children}
    </NavigationContext.Provider>
  );
}

export function useNavigation() {
  const context = useContext(NavigationContext);
  if (context === undefined) {
    throw new Error('useNavigation must be used within a NavigationProvider');
  }
  return context;
}
