// Stable external ids are the whole point of the demo: a second import of the
// same batch must report `unchanged` instead of creating six more trees.
const TREES = [
  {
    external_id: 'demo-001',
    number: 'DEMO-001',
    species: 'Quercus robur',
    planting_year: 1998,
    latitude: 54.78361,
    longitude: 9.4321,
  },
  {
    external_id: 'demo-002',
    number: 'DEMO-002',
    species: 'Tilia cordata',
    planting_year: 2003,
    latitude: 54.78402,
    longitude: 9.43385,
  },
  {
    external_id: 'demo-003',
    number: 'DEMO-003',
    species: 'Acer platanoides',
    planting_year: 2007,
    latitude: 54.78455,
    longitude: 9.43512,
  },
  {
    external_id: 'demo-004',
    number: 'DEMO-004',
    species: 'Fagus sylvatica',
    planting_year: 2011,
    latitude: 54.78511,
    longitude: 9.43644,
  },
  {
    external_id: 'demo-005',
    number: 'DEMO-005',
    species: 'Betula pendula',
    planting_year: 2014,
    latitude: 54.78566,
    longitude: 9.43771,
  },
  {
    external_id: 'demo-006',
    number: 'DEMO-006',
    species: 'Carpinus betulus',
    planting_year: 2019,
    latitude: 54.7862,
    longitude: 9.43902,
  },
]

export const MODIFIED_EXTERNAL_ID = 'demo-003'
export const MODIFIED_SPECIES = 'Tilia tomentosa'
export const DELETED_EXTERNAL_ID = 'demo-006'

const complete = (tree) => ({
  ...tree,
  description: 'Angelegt vom Demo-Plugin',
  additional_info: { source: 'demo-plugin', external_id: tree.external_id },
})

export const importBatch = () => ({ items: TREES.map(complete) })

export const modifiedBatch = () => ({
  items: TREES.filter((tree) => tree.external_id === MODIFIED_EXTERNAL_ID).map((tree) =>
    complete({ ...tree, species: MODIFIED_SPECIES }),
  ),
})
