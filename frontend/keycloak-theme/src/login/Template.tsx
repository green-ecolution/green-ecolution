import { useEffect } from 'react'
import { clsx } from 'keycloakify/tools/clsx'
import { kcSanitize } from 'keycloakify/lib/kcSanitize'
import type { TemplateProps } from 'keycloakify/login/TemplateProps'
import { getKcClsx } from 'keycloakify/login/lib/kcClsx'
import { useInitialize } from 'keycloakify/login/Template.useInitialize'
import BrandPanel from './BrandPanel'
import LanguageSelect from './components/LanguageSelect'
import type { I18n } from './i18n'
import type { KcContext } from './KcContext'

export default function Template(props: TemplateProps<KcContext, I18n>) {
  const {
    displayInfo = false,
    displayMessage = true,
    headerNode,
    socialProvidersNode = null,
    infoNode = null,
    documentTitle,
    kcContext,
    i18n,
    doUseDefaultCss,
    classes,
    children,
  } = props

  const { kcClsx } = getKcClsx({ doUseDefaultCss, classes })
  const { msg, msgStr } = i18n
  const { realm, auth, url, message, isAppInitiatedAction } = kcContext

  useEffect(() => {
    document.title = documentTitle ?? msgStr('loginTitle', realm.displayName || realm.name)
  }, [documentTitle, msgStr, realm.displayName, realm.name])

  const { isReadyToRender } = useInitialize({ kcContext, doUseDefaultCss })

  if (!isReadyToRender) {
    return null
  }

  return (
    <div className="flex min-h-dvh flex-col lg:flex-row">
      <BrandPanel variant="header" claim={msgStr('geClaim')} />
      <BrandPanel variant="side" claim={msgStr('geClaim')} />

      <main className="flex flex-1 items-center justify-center px-6 py-10">
        <div className="w-full max-w-100">
          <div className="mb-6 flex items-start justify-between gap-4">
            <h1 id="kc-page-title" className="font-[Lato] text-2xl font-bold text-foreground">
              {headerNode}
            </h1>
            <LanguageSelect i18n={i18n} />
          </div>

          {auth !== undefined && auth.showUsername && !auth.showResetCredentials && (
            <div className="mb-5 flex items-center gap-2 text-sm text-muted-foreground">
              <span id="kc-attempted-username">{auth.attemptedUsername}</span>
              <a
                id="reset-login"
                href={url.loginRestartFlowUrl}
                aria-label={msgStr('restartLoginTooltip')}
                className="font-semibold text-primary underline-offset-4 hover:underline"
              >
                {msg('restartLoginTooltip')}
              </a>
            </div>
          )}

          {displayMessage &&
            message !== undefined &&
            (message.type !== 'warning' || !isAppInitiatedAction) && (
              <div
                role="alert"
                className={clsx(
                  kcClsx('kcAlertClass'),
                  message.type === 'error' && 'border-destructive/40 bg-destructive/10 text-destructive',
                  message.type === 'warning' && 'border-yellow-200 bg-yellow-50 text-foreground',
                  message.type === 'success' && 'border-green-dark-200 bg-green-dark-50 text-foreground',
                  message.type === 'info' && 'border-border bg-muted text-foreground',
                )}
              >
                <span
                  className={kcClsx('kcAlertTitleClass')}
                  dangerouslySetInnerHTML={{ __html: kcSanitize(message.summary) }}
                />
              </div>
            )}

          {children}

          {auth !== undefined && auth.showTryAnotherWayLink && (
            <form id="kc-select-try-another-way-form" action={url.loginAction} method="post">
              <input type="hidden" name="tryAnotherWay" value="on" />
              <button
                type="submit"
                id="try-another-way"
                className="mt-4 text-sm font-semibold text-primary underline-offset-4 hover:underline"
              >
                {msg('doTryAnotherWay')}
              </button>
            </form>
          )}

          {socialProvidersNode}

          {displayInfo && (
            <div id="kc-info" className={kcClsx('kcSignUpClass')}>
              <div id="kc-info-wrapper">{infoNode}</div>
            </div>
          )}

          <p className="mt-10 text-center text-xs text-muted-foreground">
            {realm.displayName || realm.name}
          </p>
        </div>
      </main>
    </div>
  )
}
