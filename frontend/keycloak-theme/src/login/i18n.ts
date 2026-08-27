import { i18nBuilder } from 'keycloakify/login'
import type { ThemeName } from '../kc.gen'

const i18n = i18nBuilder
  .withThemeName<ThemeName>()
  .withCustomTranslations({
    de: {
      geClaim: 'Smartes Grünflächenmanagement für Kommunen',
      geOr: 'oder',
      loginAccountTitle: 'Willkommen zurück',
      doLogIn: 'Anmelden',
      doForgotPassword: 'Passwort vergessen?',
      rememberMe: 'Angemeldet bleiben',
    },
    en: {
      geClaim: 'Smart green space management for municipalities',
      geOr: 'or',
      loginAccountTitle: 'Welcome back',
      doLogIn: 'Sign in',
      doForgotPassword: 'Forgot your password?',
      rememberMe: 'Stay signed in',
    },
  })
  .build()

const { useI18n } = i18n
type I18n = typeof i18n.ofTypeI18n

export { useI18n, type I18n }
