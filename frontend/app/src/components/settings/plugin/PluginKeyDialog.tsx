import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Check, Copy, TriangleAlert } from 'lucide-react'
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  toast,
} from '@green-ecolution/ui'

interface PluginKeyDialogProps {
  open: boolean
  /** Install-time and rotate-time differ in title, and install adds the activation hint. */
  variant: 'installed' | 'rotated'
  apiKey: string
  onOpenChange: (open: boolean) => void
}

/**
 * A key is `gep_<plugin id>.<secret>`. Splitting it puts the secret on its own
 * line instead of letting a 100-character string wrap wherever it lands, and
 * lets the id line stay quiet: it only says which plugin the key belongs to.
 */
const splitKey = (apiKey: string): [string, string] => {
  const dot = apiKey.indexOf('.')
  return dot === -1 ? [apiKey, ''] : [apiKey.slice(0, dot), apiKey.slice(dot)]
}

const PluginKeyDialog = ({ open, variant, apiKey, onOpenChange }: PluginKeyDialogProps) => {
  const { t } = useTranslation('settings')
  const [copied, setCopied] = useState(false)
  const [identifier, secret] = splitKey(apiKey)

  const handleCopy = () => {
    navigator.clipboard.writeText(apiKey).then(
      () => {
        setCopied(true)
        setTimeout(() => setCopied(false), 2000)
      },
      () => {
        toast.error(t('plugin.keyDialog.copyFailed'))
      },
    )
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        // Dismissing this by accident costs the key for good: it is shown once
        // and the only way back is rotating it. Escape and the close controls
        // stay, so there is still a way out that reads as a decision.
        onInteractOutside={(event) => event.preventDefault()}
      >
        <DialogHeader>
          <DialogTitle>
            {variant === 'installed'
              ? t('plugin.keyDialog.installedTitle')
              : t('plugin.keyDialog.rotatedTitle')}
          </DialogTitle>
          <DialogDescription>
            {variant === 'installed'
              ? t('plugin.keyDialog.installedDescription')
              : t('plugin.keyDialog.rotatedDescription')}
          </DialogDescription>
        </DialogHeader>

        <div className="overflow-hidden rounded-xl border border-dark-100">
          <p className="flex items-center gap-2 border-b border-dark-100 bg-yellow-50 px-3 py-2 text-sm font-medium text-dark">
            <TriangleAlert className="size-4 shrink-0 text-yellow" aria-hidden />
            {t('plugin.keyDialog.onceNotice')}
          </p>
          <code className="block bg-dark-50 px-3 py-3 font-mono text-sm leading-relaxed sm:text-base">
            <span className="block break-all text-dark-600">{identifier}</span>
            <span className="block break-all text-dark">{secret}</span>
          </code>
        </div>

        {/* Only on install: save_new writes enabled = false, so the adapter is
            answered with 403 plugin.disabled until an admin enables it.
            Rotation leaves the flag alone, so the hint would be wrong there. */}
        {variant === 'installed' && (
          <p className="text-sm text-dark-600">{t('plugin.keyDialog.activationHint')}</p>
        )}

        <DialogFooter>
          <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
            {t('plugin.keyDialog.close')}
          </Button>
          <Button type="button" onClick={handleCopy}>
            {copied ? <Check aria-hidden /> : <Copy aria-hidden />}
            {copied ? t('plugin.keyDialog.copied') : t('plugin.keyDialog.copy')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default PluginKeyDialog
