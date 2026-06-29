export const resolveResponseStatus = (err: unknown): number | null => {
  if (err instanceof Response) return err.status
  if (
    err != null &&
    typeof err === 'object' &&
    'response' in err &&
    err.response instanceof Response
  )
    return err.response.status
  return null
}

export const mapActivateError = (err: unknown): string => {
  const status = resolveResponseStatus(err)
  if (status === 404) return 'Sensor existiert nicht (mehr). Bitte erneut versuchen.'
  if (status === 409) return 'Sensor ist bereits einem Baum zugeordnet.'
  return 'Aktivierung fehlgeschlagen. Bitte erneut versuchen.'
}

export const mapReassignError = (err: unknown): string => {
  const status = resolveResponseStatus(err)
  if (status === 404) return 'Sensor oder Baum existiert nicht (mehr).'
  if (status === 409) return 'Der Baum ist bereits einem anderen Sensor zugeordnet.'
  return 'Baumwechsel fehlgeschlagen. Bitte erneut versuchen.'
}

export const mapDeactivateError = (err: unknown): string => {
  const status = resolveResponseStatus(err)
  if (status === 404) return 'Sensor existiert nicht (mehr).'
  return 'Zurücksetzen fehlgeschlagen. Bitte erneut versuchen.'
}
