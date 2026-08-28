import { Suspense, lazy } from 'react'
import DefaultPage from 'keycloakify/login/DefaultPage'
import Template from './Template'
import type { KcContext } from './KcContext'
import { useI18n } from './i18n'
import { classes } from './classes'
import '../css/theme.css'

const UserProfileFormFields = lazy(() => import('keycloakify/login/UserProfileFormFields'))
const Login = lazy(() => import('./pages/Login'))

export default function KcPage(props: { kcContext: KcContext }) {
  const { kcContext } = props
  const { i18n } = useI18n({ kcContext })

  return (
    <Suspense>
      {(() => {
        switch (kcContext.pageId) {
          case 'login.ftl':
            return (
              <Login
                kcContext={kcContext}
                i18n={i18n}
                classes={classes}
                Template={Template}
                doUseDefaultCss={false}
              />
            )
          default:
            return (
              <DefaultPage
                kcContext={kcContext}
                i18n={i18n}
                classes={classes}
                Template={Template}
                doUseDefaultCss={false}
                UserProfileFormFields={UserProfileFormFields}
                doMakeUserConfirmPassword={true}
              />
            )
        }
      })()}
    </Suspense>
  )
}
