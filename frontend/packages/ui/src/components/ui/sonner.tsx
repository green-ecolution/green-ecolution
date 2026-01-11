'use client'

import { Check, CircleCheck, Info, TriangleAlert, X } from 'lucide-react'
import { Toaster as Sonner, toast as sonnerToast, ExternalToast } from 'sonner'

import { cn } from '@/lib/utils'

type ToasterProps = React.ComponentProps<typeof Sonner>

const DefaultIcon = () => (
  <figure
    aria-hidden="true"
    className="relative flex h-6 w-6 items-center justify-center rounded-full bg-slate-500 before:absolute before:-z-10 before:h-8 before:w-8 before:rounded-full before:bg-slate-500/50"
  >
    <CircleCheck className="h-4 w-4 text-white" />
  </figure>
)

const Toaster = ({ className, ...props }: ToasterProps) => {
  return (
    <Sonner
      className={cn('toaster group', className)}
      position="bottom-center"
      toastOptions={{
        unstyled: true,
        classNames: {
          toast:
            'w-fit bg-dark text-white rounded-[1.875rem] flex items-center gap-x-3 pr-5 pl-3 py-3',
          title: 'text-sm font-semibold',
          description: 'text-sm text-white/70 mt-0.5',
        },
      }}
      icons={{
        success: (
          <figure
            aria-hidden="true"
            className="relative flex h-6 w-6 items-center justify-center rounded-full bg-green-light before:absolute before:-z-10 before:h-8 before:w-8 before:rounded-full before:bg-green-light/50"
          >
            <Check className="h-4 w-4 text-white" />
          </figure>
        ),
        error: (
          <figure
            aria-hidden="true"
            className="relative flex h-6 w-6 items-center justify-center rounded-full bg-red before:absolute before:-z-10 before:h-8 before:w-8 before:rounded-full before:bg-red/50"
          >
            <X className="h-4 w-4 text-white" />
          </figure>
        ),
        warning: (
          <figure
            aria-hidden="true"
            className="relative flex h-6 w-6 items-center justify-center rounded-full bg-yellow before:absolute before:-z-10 before:h-8 before:w-8 before:rounded-full before:bg-yellow/50"
          >
            <TriangleAlert className="h-4 w-4 text-white" />
          </figure>
        ),
        info: (
          <figure
            aria-hidden="true"
            className="relative flex h-6 w-6 items-center justify-center rounded-full bg-slate-500 before:absolute before:-z-10 before:h-8 before:w-8 before:rounded-full before:bg-slate-500/50"
          >
            <Info className="h-4 w-4 text-white" />
          </figure>
        ),
      }}
      {...props}
    />
  )
}

const toast = Object.assign(
  (message: string, data?: ExternalToast) => {
    return sonnerToast(message, {
      ...data,
      icon: data?.icon ?? <DefaultIcon />,
    })
  },
  {
    success: sonnerToast.success,
    error: sonnerToast.error,
    warning: sonnerToast.warning,
    info: sonnerToast.info,
    loading: sonnerToast.loading,
    promise: sonnerToast.promise,
    dismiss: sonnerToast.dismiss,
    custom: sonnerToast.custom,
    message: sonnerToast.message,
  }
)

export { Toaster, toast }
