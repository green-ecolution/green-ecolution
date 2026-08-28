import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/react'
import KcPage from '../login/KcPage'
import { getKcContextMock } from '../login/KcPageStory'

const pageIds = [
  'login.ftl',
  'login-reset-password.ftl',
  'login-update-password.ftl',
  'login-page-expired.ftl',
  'logout-confirm.ftl',
  'login-idp-link-confirm.ftl',
  'login-idp-link-email.ftl',
  'error.ftl',
  'info.ftl',
] as const

describe('KcPage', () => {
  it.each(pageIds)('renders %s without throwing', (pageId) => {
    const kcContext = getKcContextMock({ pageId, overrides: {} })
    expect(() => render(<KcPage kcContext={kcContext} />)).not.toThrow()
  })

  it('renders the login form with the fields keycloak posts back', async () => {
    const kcContext = getKcContextMock({
      pageId: 'login.ftl',
      overrides: {
        url: { loginAction: '/realms/green-ecolution/login-actions/authenticate' },
        realm: { password: true, rememberMe: true, resetPasswordAllowed: true },
        social: {
          providers: [
            {
              alias: 'github',
              displayName: 'GitHub',
              loginUrl: '/realms/green-ecolution/broker/github/login',
              providerId: 'github',
              iconClasses: 'fa fa-github',
            },
          ],
        },
      },
    })

    render(<KcPage kcContext={kcContext} />)

    await screen.findByRole('button', { name: /anmelden|sign in|log in/i })

    const form = document.getElementById('kc-form-login')
    expect(form).toHaveAttribute('action', '/realms/green-ecolution/login-actions/authenticate')
    expect(document.getElementById('username')).toBeInTheDocument()
    expect(document.getElementById('password')).toBeInTheDocument()
    expect(document.getElementById('rememberMe')).toBeInTheDocument()
    expect(document.getElementById('social-github')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /passwort|password/i })).toBeInTheDocument()
  })

  it('renders the claim and the divider label, never the raw message keys', async () => {
    const kcContext = getKcContextMock({ pageId: 'login.ftl', overrides: {} })

    render(<KcPage kcContext={kcContext} />)

    await screen.findByRole('button', { name: /anmelden|sign in|log in/i })

    for (const key of [
      'geBrand',
      'geHeadline',
      'geIntro',
      'geLoginSubtitle',
      'geEmailPlaceholder',
      'gePasswordPlaceholder',
      'geOr',
      'doForgotPassword',
      'loginAccountTitle',
    ]) {
      expect(document.body.textContent).not.toContain(key)
    }

    expect(screen.getByText(/Grünflächenmanagement|green space management/i)).toBeInTheDocument()
    expect(screen.getByPlaceholderText(/beispiel\.de|example\.com/i)).toBeInTheDocument()
  })
})
