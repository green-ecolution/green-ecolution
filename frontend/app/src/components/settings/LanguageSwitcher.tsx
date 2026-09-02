import { SegmentedControl } from '@green-ecolution/ui'
import { useTranslation } from 'react-i18next'
import { SUPPORTED_LANGUAGES, languageOf } from '@/lib/i18n/languages'
import { switchLanguage } from '@/lib/i18n'
import createToast from '@/hooks/createToast'

function LanguageSwitcher() {
  const { t, i18n } = useTranslation()
  const showToast = createToast()

  const options = SUPPORTED_LANGUAGES.map((language) => ({
    value: language,
    label: t(`language.${language}`),
  }))

  const handleChange = (value: (typeof SUPPORTED_LANGUAGES)[number]) => {
    void switchLanguage(value).catch((error: unknown) => {
      console.error('Language switch failed', error)
      // Only the target chunk failed to load, so the current catalog is
      // still in place — this toast renders correctly in the previous
      // language, which is why a translated message works here.
      showToast(t('language.switchFailed'), 'error')
    })
  }

  return (
    <div className="flex flex-col gap-y-2">
      <SegmentedControl
        ariaLabel={t('language.label')}
        options={options}
        value={languageOf(i18n.language)}
        onChange={handleChange}
      />
      <p className="text-sm text-dark-600">{t('language.description')}</p>
    </div>
  )
}

export default LanguageSwitcher
