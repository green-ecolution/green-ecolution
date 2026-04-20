import * as React from 'react'
import { Copy, Check } from 'lucide-react'
import { cn } from '@/lib/utils'

export interface CopyableTextProps extends React.HTMLAttributes<HTMLDivElement> {
  /** The text value to display and copy. */
  value: string
  /** Optional label rendered above the text. */
  label?: string
  /** Callback fired after a successful copy. */
  onCopy?: () => void
  /** Callback fired when copying fails. */
  onCopyError?: () => void
}

const CopyableText = React.forwardRef<HTMLDivElement, CopyableTextProps>(
  ({ value, label, onCopy, onCopyError, className, ...props }, ref) => {
    const [copied, setCopied] = React.useState(false)

    const handleCopy = () => {
      navigator.clipboard
        .writeText(value)
        .then(() => {
          setCopied(true)
          onCopy?.()
          setTimeout(() => setCopied(false), 2000)
        })
        .catch(() => {
          onCopyError?.()
        })
    }

    return (
      <div ref={ref} className={cn('flex flex-col gap-2', className)} {...props}>
        {label && (
          <span className="text-xs uppercase tracking-widest text-muted-foreground">{label}</span>
        )}
        <code className="relative flex items-center font-mono text-lg md:text-xl font-semibold break-all bg-dark-50 rounded-lg pl-3 pr-10 py-2 border border-dark-100">
          <span className="flex-1">{value}</span>
          <button
            type="button"
            onClick={handleCopy}
            aria-label={`${label ?? value} kopieren`}
            className="absolute right-1.5 top-1/2 -translate-y-1/2 p-1.5 rounded-md text-muted-foreground hover:text-foreground hover:bg-dark-100 transition-colors cursor-pointer"
          >
            {copied ? <Check className="size-4 text-green-dark" /> : <Copy className="size-4" />}
          </button>
        </code>
      </div>
    )
  },
)
CopyableText.displayName = 'CopyableText'

export { CopyableText }
