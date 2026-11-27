export interface TreeForm {
  number: string
  species: string
  plantingYear: number
  latitude: number
  longitude: number
  description: string
  treeClusterId: string | null
  sensorId: string | null
  provider?: string
}
