import useStore from '@/store/store'
import Option from '../Option'

const ClusterFieldset = () => {
  const hasCluster = useStore((s) => s.filterDraft.hasCluster)
  const setHasCluster = useStore((s) => s.setFilterHasCluster)
  const treeClusterOptions = [
    {
      label: 'Gruppe zugehörig',
      value: true,
    },
    {
      label: 'Keiner Gruppe zugehörig',
      value: false,
    },
  ]
  return (
    <fieldset className="mt-4">
      <legend className="font-lato font-semibold text-dark-600 mb-2">
        Zugehörigkeit einer Bewässerungsgruppe:
      </legend>
      {treeClusterOptions.map((type) => (
        <Option
          key={type.label}
          label={type.label}
          name={type.value.toString()}
          value={String(type.value)}
          checked={hasCluster === type.value}
          onChange={(event) =>
            setHasCluster(event.target.checked ? event.target.value === 'true' : undefined)
          }
        />
      ))}
    </fieldset>
  )
}

export default ClusterFieldset
