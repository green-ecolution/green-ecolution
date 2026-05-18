import { useEffect, useRef, useState } from 'react'
import { Check, Copy, Eye, EyeOff, ShieldAlert } from 'lucide-react'
import { cn, toast } from '@green-ecolution/ui'
import { SECRET_MASK } from './secrets'

interface SecretRevealProps {
  label: string
  value: string
  /** Seconds the value stays revealed after toggling on. Default 10. */
  autoHideSeconds?: number
}

const SecretReveal = ({ label, value, autoHideSeconds = 10 }: SecretRevealProps) => {
  const [revealed, setRevealed] = useState(false)
  const [copied, setCopied] = useState(false)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => {
    if (!revealed) return
    timerRef.current = setTimeout(() => setRevealed(false), autoHideSeconds * 1000)
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current)
    }
  }, [revealed, autoHideSeconds])

  const handleCopy = () => {
    navigator.clipboard.writeText(value).then(
      () => {
        setCopied(true)
        setTimeout(() => setCopied(false), 2000)
      },
      () => {
        toast.error('Kopieren fehlgeschlagen.')
      },
    )
  }

  return (
    <div className="flex flex-col gap-2">
      <span className="text-xs uppercase tracking-widest text-muted-foreground flex items-center gap-1.5">
        {label}
        <ShieldAlert className="size-3 text-yellow" aria-label="Sensible Daten" />
      </span>
      <code className="relative flex items-center font-mono text-lg md:text-xl font-semibold break-all bg-dark-50 rounded-lg pl-3 pr-20 py-2 border border-dark-100 min-h-12">
        <span
          className={cn('flex-1', !revealed && 'tracking-[0.2em] text-dark-600 select-none')}
          aria-live="polite"
        >
          {revealed ? value : SECRET_MASK}
        </span>
        <div className="absolute right-1.5 top-1/2 -translate-y-1/2 flex gap-0.5">
          <button
            type="button"
            onClick={() => setRevealed((v) => !v)}
            aria-label={revealed ? `${label} verbergen` : `${label} anzeigen`}
            aria-pressed={revealed}
            className="p-1.5 rounded-md text-muted-foreground hover:text-foreground hover:bg-dark-100 transition-colors cursor-pointer"
          >
            {revealed ? <EyeOff className="size-4" /> : <Eye className="size-4" />}
          </button>
          <button
            type="button"
            onClick={handleCopy}
            aria-label={`${label} kopieren`}
            className="p-1.5 rounded-md text-muted-foreground hover:text-foreground hover:bg-dark-100 transition-colors cursor-pointer"
          >
            {copied ? <Check className="size-4 text-green-dark" /> : <Copy className="size-4" />}
          </button>
        </div>
      </code>
      {revealed && (
        <p className="text-xs text-muted-foreground">
          Wird in {autoHideSeconds}s automatisch verborgen.
        </p>
      )}
    </div>
  )
}

export default SecretReveal
