import * as React from 'react'

import { cn } from '@/lib/utils'

const Textarea = React.forwardRef<HTMLTextAreaElement, React.ComponentProps<'textarea'>>(
  ({ className, ...props }, ref) => {
    return (
      <textarea
        data-slot="textarea"
        className={cn(
          'flex min-h-[80px] w-full resize-y rounded-lg border border-dark-200 bg-white px-3 py-2 text-base text-dark-800 shadow-xs placeholder:text-dark-400 transition-[color,box-shadow] outline-none focus-visible:border-green-dark focus-visible:ring-green-dark/50 focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50 md:text-sm',
          className,
        )}
        ref={ref}
        {...props}
      />
    )
  },
)
Textarea.displayName = 'Textarea'

export { Textarea }
