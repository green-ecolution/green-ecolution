/**
 * Every user-facing text for a failed backend call, keyed by a stable key.
 *
 * Keys mirror what the response carries: `code.<code>` when the body names a
 * cause, `status.<http status>` otherwise. Nothing else in the app formats a
 * backend error, so introducing i18n means pointing `messageFor` at `t(key)`
 * and moving these strings into the locale files — the keys stay as they are.
 */
export type ApiErrorMessageKey =
  | `code.${string}`
  | `status.${number}`
  | 'status.unknown'
  | 'offline'
  | 'unknown'
  /** Backend prose used verbatim because no catalog entry matched. */
  | 'detail'

export const apiErrorMessages: Record<string, string> = {
  // Cross-aggregate organization rules. The backend sends these codes with 422.
  'code.organization_mismatch.trees_vs_cluster':
    'Die ausgewählten Bäume gehören zu einer anderen Organisation als die Bewässerungsgruppe.',
  'code.organization_mismatch.cluster_vs_tree':
    'Die ausgewählte Bewässerungsgruppe gehört zu einer anderen Organisation als der Baum.',
  'code.organization_mismatch.sensor_vs_tree':
    'Der Sensor gehört zu einer anderen Organisation als der Baum.',
  'code.organization_mismatch.clusters_vs_plan':
    'Die ausgewählten Bewässerungsgruppen liegen außerhalb der Organisation des Einsatzplans.',
  'code.organization_mismatch.role_vs_user':
    'Die Rolle gehört zu einer anderen Organisation als die ausgewählte Person.',

  'status.400': 'Die Anfrage war fehlerhaft und konnte nicht verarbeitet werden.',
  'status.401': 'Die Anmeldung ist abgelaufen. Bitte melde dich erneut an.',
  'status.403': 'Für diese Aktion fehlt dir die Berechtigung.',
  'status.404': 'Der angeforderte Datensatz wurde nicht gefunden.',
  'status.409': 'Der Datensatz existiert bereits oder wurde zwischenzeitlich geändert.',
  'status.415': 'Das Format der Anfrage wird nicht unterstützt.',
  'status.422': 'Die eingegebenen Daten sind unvollständig oder ungültig.',
  'status.500':
    'Auf dem Server ist ein unerwarteter Fehler aufgetreten. Bitte probiere es erneut oder wende dich an einen Systemadministrierenden.',
  'status.502': 'Ein nachgelagerter Dienst hat nicht geantwortet.',
  'status.503': 'Der Dienst ist derzeit nicht erreichbar. Bitte versuche es später erneut.',
  'status.unknown': 'Der Server hat mit Status {status} geantwortet.',

  offline: 'Der Server ist nicht erreichbar. Bitte prüfe deine Verbindung.',
  unknown: 'Unbekannter Fehler',
}

/** Catalog lookup with `{name}` placeholder substitution; undefined when unknown. */
export function messageFor(
  key: ApiErrorMessageKey,
  params?: Record<string, string | number>,
): string | undefined {
  const template = apiErrorMessages[key]
  if (!template || !params) return template
  return Object.entries(params).reduce(
    (text, [name, value]) => text.split(`{${name}}`).join(String(value)),
    template,
  )
}
