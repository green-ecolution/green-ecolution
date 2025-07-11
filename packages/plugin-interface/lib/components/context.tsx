import { createContext, useContext } from 'react'

export interface PluginContext {
  authToken: string
}

export const PluginContext = createContext<PluginContext | undefined>(undefined)

export interface PluginProviderProps extends React.PropsWithChildren {
  authToken: string
}

export const PluginProvider = ({ authToken, children }: PluginProviderProps) => {
  return <PluginContext.Provider value={{ authToken }}>{children}</PluginContext.Provider>
}

export const usePluginContext = () => {
  const context = useContext(PluginContext)
  if (context === undefined) {
    throw new Error('usePluginContext must be used within a PluginProvider')
  }
  return context
}
