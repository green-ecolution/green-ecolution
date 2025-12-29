import { ReactElement, ReactNode } from 'react'
import { render, RenderOptions } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { FormProvider, useForm, FieldValues, DefaultValues } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { ZodSchema } from 'zod'
import ToastProvider from '@/context/ToastContext'

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
      <ToastProvider>{children}</ToastProvider>
    </QueryClientProvider>
  )
}

function customRender(ui: ReactElement, options?: Omit<RenderOptions, 'wrapper'>) {
  return render(ui, { wrapper: AllProviders, ...options })
}

interface FormWrapperProps<T extends FieldValues> {
  children: ReactNode
  schema: ZodSchema<T>
  defaultValues?: DefaultValues<T>
  onSubmit?: (data: T) => void
}

export function FormWrapper<T extends FieldValues>({
  children,
  schema,
  defaultValues,
  onSubmit = () => {},
}: FormWrapperProps<T>) {
  const methods = useForm<T>({
    defaultValues,
    resolver: zodResolver(schema),
  })

  return (
    <FormProvider {...methods}>
      <form onSubmit={methods.handleSubmit(onSubmit)}>{children}</form>
    </FormProvider>
  )
}

export * from '@testing-library/react'
export { default as userEvent } from '@testing-library/user-event'
export { customRender as render }
