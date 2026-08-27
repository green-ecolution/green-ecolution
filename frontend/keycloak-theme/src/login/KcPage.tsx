import { Suspense, lazy } from 'react'
import DefaultPage from 'keycloakify/login/DefaultPage'
import Template from 'keycloakify/login/Template'
import type { KcContext } from './KcContext'
import { useI18n } from './i18n'

const UserProfileFormFields = lazy(() => import('keycloakify/login/UserProfileFormFields'))

export default function KcPage(props: { kcContext: KcContext }) {
  const { kcContext } = props
  const { i18n } = useI18n({ kcContext })

  return (
    <Suspense>
      <DefaultPage
        kcContext={kcContext}
        i18n={i18n}
        Template={Template}
        doUseDefaultCss={true}
        UserProfileFormFields={UserProfileFormFields}
        doMakeUserConfirmPassword={true}
      />
    </Suspense>
  )
}
