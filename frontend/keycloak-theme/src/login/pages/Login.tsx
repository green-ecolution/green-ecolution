import { useState } from 'react'
import { kcSanitize } from 'keycloakify/lib/kcSanitize'
import { clsx } from 'keycloakify/tools/clsx'
import type { PageProps } from 'keycloakify/login/pages/PageProps'
import { getKcClsx } from 'keycloakify/login/lib/kcClsx'
import PasswordInput from '../components/PasswordInput'
import type { KcContext } from '../KcContext'
import type { I18n } from '../i18n'

export default function Login(props: PageProps<Extract<KcContext, { pageId: 'login.ftl' }>, I18n>) {
  const { kcContext, i18n, doUseDefaultCss, Template, classes } = props
  const { kcClsx } = getKcClsx({ doUseDefaultCss, classes })

  const { social, realm, url, usernameHidden, login, auth, messagesPerField } = kcContext
  const { msg, msgStr } = i18n

  const [isLoginButtonDisabled, setIsLoginButtonDisabled] = useState(false)
  const hasCredentialError = messagesPerField.existsError('username', 'password')

  return (
    <Template
      kcContext={kcContext}
      i18n={i18n}
      doUseDefaultCss={doUseDefaultCss}
      classes={classes}
      displayMessage={!hasCredentialError}
      headerNode={msg('loginAccountTitle')}
      socialProvidersNode={
        realm.password && social?.providers !== undefined && social.providers.length !== 0 ? (
          <div id="kc-social-providers" className={kcClsx('kcFormSocialAccountSectionClass')}>
            <div className="my-6 flex items-center gap-3">
              <span className="h-px flex-1 bg-border" />
              <span className="text-xs font-semibold tracking-wide text-muted-foreground uppercase">
                {msg('geOr')}
              </span>
              <span className="h-px flex-1 bg-border" />
            </div>
            <ul className={kcClsx('kcFormSocialAccountListClass')}>
              {social.providers.map((p) => (
                <li key={p.alias}>
                  <a
                    id={`social-${p.alias}`}
                    href={p.loginUrl}
                    className={kcClsx('kcFormSocialAccountListButtonClass')}
                  >
                    <span
                      className={kcClsx('kcFormSocialAccountNameClass')}
                      dangerouslySetInnerHTML={{ __html: kcSanitize(p.displayName) }}
                    />
                  </a>
                </li>
              ))}
            </ul>
          </div>
        ) : null
      }
    >
      {realm.password && (
        <form
          id="kc-form-login"
          action={url.loginAction}
          method="post"
          onSubmit={() => {
            setIsLoginButtonDisabled(true)
            return true
          }}
        >
          {!usernameHidden && (
            <div className={kcClsx('kcFormGroupClass')}>
              <label htmlFor="username" className={kcClsx('kcLabelClass')}>
                {!realm.loginWithEmailAllowed
                  ? msg('username')
                  : !realm.registrationEmailAsUsername
                    ? msg('usernameOrEmail')
                    : msg('email')}
              </label>
              <input
                id="username"
                name="username"
                type="text"
                autoFocus
                autoComplete="username"
                defaultValue={login.username ?? ''}
                className={kcClsx('kcInputClass')}
                aria-invalid={hasCredentialError}
                aria-describedby={hasCredentialError ? 'input-error' : undefined}
              />
            </div>
          )}

          <div className={kcClsx('kcFormGroupClass')}>
            <label htmlFor="password" className={kcClsx('kcLabelClass')}>
              {msg('password')}
            </label>
            <PasswordInput kcClsx={kcClsx} i18n={i18n} passwordInputId="password">
              <input
                id="password"
                name="password"
                type="password"
                autoComplete="current-password"
                className={clsx(kcClsx('kcInputClass'), 'pr-11')}
                aria-invalid={hasCredentialError}
                aria-describedby={hasCredentialError ? 'input-error' : undefined}
              />
            </PasswordInput>
          </div>

          {hasCredentialError && (
            <p
              id="input-error"
              role="alert"
              className={clsx(kcClsx('kcInputErrorMessageClass'), 'mb-5')}
              dangerouslySetInnerHTML={{
                __html: kcSanitize(messagesPerField.getFirstError('username', 'password')),
              }}
            />
          )}

          <div className={clsx(kcClsx('kcFormSettingClass'), 'mb-6')}>
            {realm.rememberMe && !usernameHidden && (
              <label className="flex items-center gap-2 text-sm text-foreground">
                <input
                  id="rememberMe"
                  name="rememberMe"
                  type="checkbox"
                  defaultChecked={!!login.rememberMe}
                  className={kcClsx('kcCheckboxInputClass')}
                />
                {msg('rememberMe')}
              </label>
            )}
            {realm.resetPasswordAllowed && (
              <a
                href={url.loginResetCredentialsUrl}
                className="ml-auto text-sm font-semibold text-primary underline-offset-4 hover:underline"
              >
                {msg('doForgotPassword')}
              </a>
            )}
          </div>

          <input
            type="hidden"
            id="id-hidden-input"
            name="credentialId"
            value={auth.selectedCredential}
          />
          <button
            id="kc-login"
            name="login"
            type="submit"
            disabled={isLoginButtonDisabled}
            className={kcClsx(
              'kcButtonClass',
              'kcButtonPrimaryClass',
              'kcButtonBlockClass',
              'kcButtonLargeClass',
            )}
          >
            {msgStr('doLogIn')}
          </button>
        </form>
      )}
    </Template>
  )
}
