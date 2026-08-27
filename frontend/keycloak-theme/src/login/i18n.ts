import { i18nBuilder } from 'keycloakify/login'
import type { ThemeName } from '../kc.gen'

const { useI18n, ofTypeI18n } = i18nBuilder
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

type I18n = typeof ofTypeI18n

export { useI18n, type I18n }
