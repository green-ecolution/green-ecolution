import type { ValidationIssue } from './types'

type Renderer = (params: Record<string, string | number>) => string

const messages: Record<string, Renderer> = {
  // Tree
  'tree.species.empty': () => 'Art ist erforderlich.',
  'tree.species.tooShort': (p) => `Art muss mindestens ${p.min} Zeichen lang sein.`,
  'tree.species.tooLong': (p) => `Art darf maximal ${p.max} Zeichen lang sein.`,
  'tree.number.empty': () => 'Baumnummer ist erforderlich.',
  'tree.number.tooShort': (p) => `Baumnummer muss mindestens ${p.min} Zeichen lang sein.`,
  'tree.number.tooLong': (p) => `Baumnummer darf maximal ${p.max} Zeichen lang sein.`,
  'tree.planting_year.outOfRange': (p) =>
    `Pflanzjahr muss zwischen ${p.min} und ${p.max} liegen.`,

  // Coordinate
  'coordinate.latitude.outOfRange': (p) =>
    `Breitengrad muss zwischen ${p.min} und ${p.max} liegen (war ${p.got}).`,
  'coordinate.longitude.outOfRange': (p) =>
    `Längengrad muss zwischen ${p.min} und ${p.max} liegen (war ${p.got}).`,

  // Cluster
  'cluster.name.empty': () => 'Name ist erforderlich.',
  'cluster.name.tooShort': (p) => `Name muss mindestens ${p.min} Zeichen lang sein.`,
  'cluster.name.tooLong': (p) => `Name darf maximal ${p.max} Zeichen lang sein.`,
  'cluster.address.empty': () => 'Adresse ist erforderlich.',
  'cluster.address.tooShort': (p) => `Adresse muss mindestens ${p.min} Zeichen lang sein.`,
  'cluster.address.tooLong': (p) => `Adresse darf maximal ${p.max} Zeichen lang sein.`,

  // Vehicle
  'vehicle.number_plate.empty': () => 'Kennzeichen ist erforderlich.',
  'vehicle.number_plate.tooShort': (p) =>
    `Kennzeichen muss mindestens ${p.min} Zeichen lang sein.`,
  'vehicle.number_plate.tooLong': (p) =>
    `Kennzeichen darf maximal ${p.max} Zeichen lang sein.`,
  'vehicle.model.empty': () => 'Modell ist erforderlich.',
  'vehicle.model.tooShort': (p) => `Modell muss mindestens ${p.min} Zeichen lang sein.`,
  'vehicle.model.tooLong': (p) => `Modell darf maximal ${p.max} Zeichen lang sein.`,
  'water_capacity.outOfRange': (p) =>
    `Wasserkapazität muss zwischen ${p.min} und ${p.max} Liter liegen.`,
  'vehicle.dimension.height.outOfRange': () => 'Höhe ist erforderlich.',
  'vehicle.dimension.width.outOfRange': () => 'Breite ist erforderlich.',
  'vehicle.dimension.length.outOfRange': () => 'Länge ist erforderlich.',
  'vehicle.dimension.weight.outOfRange': () => 'Gewicht ist erforderlich.',

  // Watering plan
  'watering_plan.cluster_ids.empty': () =>
    'Es muss mindestens ein Cluster ausgewählt werden.',
  'watering_plan.driver_ids.empty': () =>
    'Es muss mindestens ein Mitarbeiter ausgewählt werden.',
  'watering_plan.transporter_id.empty': () => 'Es muss ein Transportfahrzeug ausgewählt werden.',
  'watering_plan.date.outOfRange': () => 'Datum muss heute oder in der Zukunft liegen.',

  // User
  'user.email.empty': () => 'E-Mail ist erforderlich.',
  'user.email.invalidFormat': () => 'E-Mail-Adresse ist ungültig.',
  'user.username.empty': () => 'Benutzername ist erforderlich.',
}

export function translateIssue(issue: ValidationIssue): string {
  const renderer = messages[issue.key]
  if (!renderer) return issue.key
  return renderer(issue.params)
}
