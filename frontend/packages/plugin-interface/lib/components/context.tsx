import {
  createContext,
  useContext,
  useEffect,
  useState,
  type PropsWithChildren,
  type ReactNode,
} from 'react'
import { connectToHost, type PluginContext } from '../handshake'

const Context = createContext<PluginContext | undefined>(undefined)

export interface PluginProviderProps extends PropsWithChildren {
  /** Rendered while the handshake is still running. */
  pending?: ReactNode
  /** Rendered when the host never answered. */
  fallback?: (error: Error) => ReactNode
}

/**
 * Renders children only once the host has answered the handshake, so a
 * consumer calling usePluginContext() always gets the real context rather
 * than having to handle a not-yet-connected state itself.
 *
 * A handshake that never completes is reported instead of left as a blank
 * page: opening the plugin's URL directly, rather than inside the host, is a
 * mistake a developer makes constantly and it should say so.
 */
export const PluginProvider = ({ children, pending, fallback }: PluginProviderProps) => {
  const [context, setContext] = useState<PluginContext>()
  const [error, setError] = useState<Error>()

  useEffect(() => {
    let cancelled = false
    void connectToHost().then(
      (received) => {
        if (!cancelled) setContext(received)
      },
      (reason: unknown) => {
        if (!cancelled) setError(reason instanceof Error ? reason : new Error(String(reason)))
      },
    )
    return () => {
      cancelled = true
    }
  }, [])

  if (error !== undefined) {
    return fallback ? fallback(error) : <p role="alert">{error.message}</p>
  }
  if (context === undefined) return pending ?? null

  return <Context.Provider value={context}>{children}</Context.Provider>
}

export const usePluginContext = () => {
  const context = useContext(Context)
  if (context === undefined) {
    throw new Error('usePluginContext must be used within a PluginProvider')
  }
  return context
}
