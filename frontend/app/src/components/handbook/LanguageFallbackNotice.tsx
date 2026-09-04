import { useTranslation } from 'react-i18next'
import { Alert, AlertContent, AlertIcon } from '@green-ecolution/ui'
import { handbookIndex } from '@/lib/handbook'
import { languageOf } from '@/lib/i18n/languages'

function LanguageFallbackNotice({ className = '' }: { className?: string }) {
  const { t, i18n } = useTranslation('help')

  if (languageOf(i18n.language) === handbookIndex.language) return null

  return (
    <Alert variant="info" className={`w-full flex gap-3 ${className}`}>
      <AlertIcon variant="info" />
      <AlertContent className="text-sm leading-relaxed">{t('chapter.fallbackNotice')}</AlertContent>
    </Alert>
  )
}

export default LanguageFallbackNotice
