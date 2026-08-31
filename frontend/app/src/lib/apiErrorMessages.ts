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

  // Conflicts with stored state. All of these arrive as 409, so without the
  // code they would collapse into one wrong sentence.
  'code.conflict.tree_already_has_sensor': 'An diesem Baum hängt bereits ein anderer Sensor.',
  'code.conflict.sensor_already_assigned':
    'Dieser Sensor ist bereits einem anderen Baum zugeordnet.',
  'code.conflict.sensor_already_activated': 'Dieser Sensor ist bereits aktiviert.',
  'code.conflict.sensor_not_activated': 'Dieser Sensor ist noch nicht aktiviert.',
  'code.conflict.sensor_bound_to_tree':
    'Der Sensor hängt an einem Baum und kann nur zusammen mit ihm übertragen werden.',
  'code.conflict.tree_in_cluster':
    'Der Baum gehört zu einer Bewässerungsgruppe und muss erst daraus entfernt werden.',
  'code.conflict.organization_not_empty':
    'Die Organisation hat noch Unterorganisationen oder Mitglieder.',
  'code.conflict.root_organization_immutable':
    'Die oberste Organisation kann nicht geändert oder gelöscht werden.',
  'code.conflict.role_template_immutable':
    'Vorlagen-Rollen lassen sich nicht ändern. Lege stattdessen eine Kopie in deiner Organisation an.',
  'code.conflict.role_template_not_assignable':
    'Vorlagen-Rollen können keiner Person zugewiesen werden.',
  'code.conflict.cannot_change_own_access':
    'Du kannst deine eigenen Rollen und deine eigene Organisation nicht ändern.',
  'code.conflict.cannot_revoke_own_administration':
    'Diese Änderung würde dir selbst das Recht nehmen, Rollen und Mitglieder zu verwalten.',

  'code.organization.missing':
    'Es wurde keine Organisation angegeben und dein Konto gehört zu keiner.',
  'code.organization.contact_person_not_a_member':
    'Die ausgewählte Ansprechperson gehört nicht zu dieser Organisation.',

  // Watering plan state machine. All 400, only the code tells them apart.
  'code.watering_plan.invalid_state_transition':
    'Dieser Statuswechsel ist im aktuellen Zustand des Einsatzplans nicht möglich.',
  'code.watering_plan.cannot_mutate_after_start':
    'Der Einsatzplan lässt sich nur bearbeiten, solange er noch nicht gestartet ist.',
  'code.watering_plan.cancellation_note_required': 'Für den Abbruch wird eine Begründung benötigt.',
  'code.watering_plan.evaluation_missing_for_cluster':
    'Für jede zugewiesene Bewässerungsgruppe wird eine Auswertung benötigt.',

  'code.feature.routing_disabled': 'Die Routenplanung ist in dieser Umgebung nicht aktiviert.',
  'code.feature.plugins_disabled': 'Das Plugin-System ist in dieser Umgebung nicht aktiviert.',

  'code.routing.unavailable': 'Der Routing-Dienst ist derzeit nicht erreichbar.',
  'code.routing.invalid_problem':
    'Aus den gewählten Angaben lässt sich keine Route berechnen. Prüfe Fahrzeug, Startpunkt und Gruppen.',
  'code.routing.failed': 'Die Routenberechnung ist fehlgeschlagen.',

  // Request parts the backend could not parse. Each names a different input,
  // so a form can point at the right control.
  'code.request.malformed_date': 'Das Datum konnte nicht gelesen werden.',
  'code.request.malformed_uuid': 'Ein ausgewählter Eintrag hat keine gültige Kennung.',
  'code.request.malformed_sensor_id': 'Die Sensor-Kennung hat kein gültiges Format.',
  'code.request.malformed_bounding_box': 'Der Kartenausschnitt konnte nicht gelesen werden.',
  'code.request.unknown_permission': 'Die Anfrage nennt eine unbekannte Berechtigung.',

  // Client-side defects rather than user mistakes. They still need their own
  // text: `status.404` would claim a record was not found, when in truth the
  // endpoint does not exist, which usually means app and server versions differ.
  'code.request.unknown_endpoint':
    'Diese Funktion kennt der Server nicht. Vermutlich passen App und Server nicht zusammen. Lade die Seite neu.',
  'code.request.method_not_allowed': 'Die Anfrage ist für diesen Endpunkt nicht zulässig.',
  'code.request.malformed_path_parameter': 'Der Verweis in der Adresse ist ungültig.',
  'code.request.malformed_query_parameter': 'Ein Filter- oder Seitenparameter ist ungültig.',
  'code.request.malformed_body': 'Die gesendeten Daten konnten nicht gelesen werden.',

  // Deliberately absent: `resource.*`, `auth.*`, `request.invalid_input`,
  // `request.validation_failed` and `internal.error`. The first three and the
  // last say no more than their status does; `request.validation_failed`
  // always ships a `validation` block, whose own key produces the better text.

  'status.400': 'Die Anfrage war fehlerhaft und konnte nicht verarbeitet werden.',
  'status.401': 'Die Anmeldung ist abgelaufen. Bitte melde dich erneut an.',
  'status.403': 'Für diese Aktion fehlt dir die Berechtigung.',
  'status.404': 'Der angeforderte Datensatz wurde nicht gefunden.',
  'status.409': 'Der Datensatz existiert bereits oder wurde zwischenzeitlich geändert.',
  'status.405': 'Die Anfrage ist für diesen Endpunkt nicht zulässig.',
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
