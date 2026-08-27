import { createGetKcContextMock } from 'keycloakify/login/KcContext'
import { themeNames, kcEnvDefaults } from '../kc.gen'
import type { KcContextExtension, KcContextExtensionPerPage } from './KcContext'

const kcContextExtension: KcContextExtension = {
  themeName: themeNames[0],
  properties: { ...kcEnvDefaults },
}

const kcContextExtensionPerPage: KcContextExtensionPerPage = {}

export const { getKcContextMock } = createGetKcContextMock({
  kcContextExtension,
  kcContextExtensionPerPage,
  overrides: {},
  overridesPerPage: {},
})
