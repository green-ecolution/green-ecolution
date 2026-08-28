import { i18nBuilder } from 'keycloakify/login'
import type { ThemeName } from '../kc.gen'

const i18n = i18nBuilder
  .withThemeName<ThemeName>()
  .withCustomTranslations({
    de: {
      geBrand: 'Green Ecolution',
      geHeadline: 'Willkommen beim smarten Grünflächenmanagement',
      geIntro:
        'Melde Dich an, um Sensordaten Deiner Bäume zu prüfen, Bewässerungsgruppen zu planen und Einsatztouren zu koordinieren.',
      geLoginSubtitle: 'Melde Dich mit Deinem Konto an, um zum Dashboard zu gelangen.',
      geEmailPlaceholder: 'name@beispiel.de',
      gePasswordPlaceholder: 'Dein Passwort',
      geOr: 'oder',
      loginAccountTitle: 'Anmelden',
      doLogIn: 'Anmelden',
      doForgotPassword: 'Passwort vergessen?',
      rememberMe: 'Angemeldet bleiben',
    },
    en: {
      geBrand: 'Green Ecolution',
      geHeadline: 'Welcome to smart green space management',
      geIntro:
        'Sign in to check sensor data for your trees, plan watering groups and coordinate field routes.',
      geLoginSubtitle: 'Sign in with your account to reach the dashboard.',
      geEmailPlaceholder: 'name@example.com',
      gePasswordPlaceholder: 'Your password',
      geOr: 'or',
      loginAccountTitle: 'Sign in',
      doLogIn: 'Sign in',
      doForgotPassword: 'Forgot your password?',
      rememberMe: 'Stay signed in',
    },
  })
  .build()

const { useI18n } = i18n
type I18n = typeof i18n.ofTypeI18n

export { useI18n, type I18n }
