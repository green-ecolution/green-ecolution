import * as React from 'react'
import { Slot } from '@radix-ui/react-slot'
import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '@/lib/utils'

const buttonVariants = cva(
  'group inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-lg text-base font-medium cursor-pointer transition-[color,background-color,border-color,box-shadow,opacity,scale] duration-quick ease-out motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-6 [&_svg]:shrink-0',
  {
    variants: {
      // Surface variants take the press scale; text-only variants dim instead,
      // since scaling a bare label reads as a rendering glitch.
      variant: {
        default: 'bg-primary text-primary-foreground hover:bg-secondary active:scale-[0.97]',
        destructive:
          'bg-destructive text-destructive-foreground hover:bg-destructive/90 active:scale-[0.97]',
        outline:
          'border border-dark-600 bg-background text-dark-600 hover:border-dark hover:text-dark active:scale-[0.97]',
        secondary:
          'bg-secondary text-secondary-foreground hover:bg-secondary/80 active:scale-[0.97]',
        ghost: 'hover:bg-accent hover:text-accent-foreground active:scale-[0.97]',
        'ghost-destructive':
          'text-destructive hover:bg-destructive/10 hover:text-destructive active:scale-[0.97]',
        link: 'text-primary underline-offset-4 hover:underline active:opacity-70',
        'link-destructive': 'text-destructive underline-offset-4 hover:underline active:opacity-70',
        nav: 'text-primary active:opacity-70',
      },
      size: {
        default: 'h-10 px-4 py-2',
        sm: 'h-9 px-3',
        lg: 'h-11 px-8',
        icon: 'h-10 w-10',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  },
)

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>, VariantProps<typeof buttonVariants> {
  asChild?: boolean
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : 'button'
    return (
      <Comp className={cn(buttonVariants({ variant, size, className }))} ref={ref} {...props} />
    )
  },
)
Button.displayName = 'Button'

export { Button, buttonVariants }
