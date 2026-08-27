/* eslint-disable @typescript-eslint/no-empty-object-type */
import type { ExtendKcContext } from 'keycloakify/login'
import type { KcEnvName, ThemeName } from '../kc.gen'

export interface KcContextExtension {
  themeName: ThemeName
  properties: Record<KcEnvName, string> & {}
}

// eslint-disable-next-line @typescript-eslint/consistent-type-definitions -- an empty interface doesn't get an implicit index signature, so ExtendKcContext's `Record<string, Record<string, unknown>>` constraint rejects it; must stay a type alias
export type KcContextExtensionPerPage = {}

export type KcContext = ExtendKcContext<KcContextExtension, KcContextExtensionPerPage>
