/* eslint-disable react-refresh/only-export-components */
import { ReactElement, ReactNode } from 'react'
import { render, renderHook, RenderOptions } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { Toaster } from '@green-ecolution/ui'

function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
        staleTime: 0,
      },
      mutations: {
        retry: false,
      },
    },
  })
}

interface ProvidersProps {
  children: ReactNode
}

function AllProviders({ children }: ProvidersProps) {
  const queryClient = createTestQueryClient()
  return (
    <QueryClientProvider client={queryClient}>
      {children}
      <Toaster />
    </QueryClientProvider>
  )
}

function customRender(ui: ReactElement, options?: Omit<RenderOptions, 'wrapper'>) {
  return render(ui, { wrapper: AllProviders, ...options })
}

function renderHookWithClient<TResult>(callback: () => TResult) {
  const queryClient = createTestQueryClient()
  const wrapper = ({ children }: ProvidersProps) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  )
  return { ...renderHook(callback, { wrapper }), queryClient }
}

export * from '@testing-library/react'
export { default as userEvent } from '@testing-library/user-event'
export { customRender as render, renderHookWithClient }
