export const uiDe = {
  dialog: { close: 'Schließen' },
  breadcrumb: { label: 'Seitennavigation', more: 'Mehr' },
  pagination: {
    label: 'Seitennummerierung',
    previous: 'Zurück',
    next: 'Weiter',
    morePages: 'Weitere Seiten',
    goToPrevious: 'Zur vorherigen Seite',
    goToNext: 'Zur nächsten Seite',
    previousPage: 'Vorherige Seite',
    nextPage: 'Nächste Seite',
  },
  calendar: {
    previousMonth: 'Vorheriger Monat',
    nextMonth: 'Nächster Monat',
    selectMonth: 'Monat auswählen',
    selectYear: 'Jahr auswählen',
  },
  combobox: {
    placeholder: 'Auswählen…',
    searchPlaceholder: 'Suchen…',
    empty: 'Keine Treffer.',
    selectedCount: '{count} ausgewählt',
  },
  datePicker: { placeholder: 'Datum auswählen' },
  fileUpload: { remove: 'Datei entfernen', prompt: 'Klicken zum Hochladen' },
  statusCard: { moreInfo: 'Weitere Informationen' },
  stepper: { progress: 'Fortschritt' },
  timeRange: { label: 'Zeitraum' },
  camera: { preview: 'Kamera-Vorschau' },
  accuracy: {
    excellent: 'Sehr gut',
    good: 'Gut',
    fair: 'Mäßig',
    poor: 'Ungenau',
    searching: 'Suche …',
  },
} as const

/**
 * The catalog shape with widened value types.
 *
 * Must NOT be `typeof uiDe`: with `as const` that type carries the German
 * literals, so `fallbackTranslate(uiEn)` would fail to type-check because
 * `'Close'` is not assignable to `'Schließen'`.
 */
export type UiCatalog = {
  [K in keyof typeof uiDe]: { [P in keyof (typeof uiDe)[K]]: string }
}

export const uiEn = {
  dialog: { close: 'Close' },
  breadcrumb: { label: 'Breadcrumb', more: 'More' },
  pagination: {
    label: 'Pagination',
    previous: 'Back',
    next: 'Next',
    morePages: 'More pages',
    goToPrevious: 'Go to the previous page',
    goToNext: 'Go to the next page',
    previousPage: 'Previous page',
    nextPage: 'Next page',
  },
  calendar: {
    previousMonth: 'Previous month',
    nextMonth: 'Next month',
    selectMonth: 'Select month',
    selectYear: 'Select year',
  },
  combobox: {
    placeholder: 'Select…',
    searchPlaceholder: 'Search…',
    empty: 'No matches.',
    selectedCount: '{count} selected',
  },
  datePicker: { placeholder: 'Select date' },
  fileUpload: { remove: 'Remove file', prompt: 'Click to upload' },
  statusCard: { moreInfo: 'More information' },
  stepper: { progress: 'Progress' },
  timeRange: { label: 'Time range' },
  camera: { preview: 'Camera preview' },
  accuracy: {
    excellent: 'Excellent',
    good: 'Good',
    fair: 'Fair',
    poor: 'Poor',
    searching: 'Searching …',
  },
} as const satisfies UiCatalog

/** Dotted paths through the catalog, e.g. `'pagination.goToNext'`. */
export type UiTextKey = Paths<UiCatalog>

type Paths<T> = {
  [K in keyof T & string]: T[K] extends string ? K : `${K}.${Paths<T[K]>}`
}[keyof T & string]

export type UiTranslate = (key: UiTextKey, params?: Record<string, string | number>) => string
