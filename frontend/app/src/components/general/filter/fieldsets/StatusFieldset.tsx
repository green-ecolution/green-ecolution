import useStore from '@/store/store'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { WateringStatus } from '@green-ecolution/backend-client'
import { MultiSelectCombobox } from '@green-ecolution/ui'
import SelectedTagList from '../SelectedTagList'

const STATUS_OPTIONS = Object.values(WateringStatus).map((value) => ({
  value,
  label: getWateringStatusDetails(value).label,
}))

const StatusFieldset = () => {
  const statusTags = useStore((s) => s.filterDraft.statusTags)
  const setStatusTags = useStore((s) => s.setFilterStatusTags)

  return (
    <fieldset>
      <legend className="font-lato font-semibold text-dark-600 mb-2">
        Zustand der Bewässerung:
      </legend>
      <MultiSelectCombobox
        options={STATUS_OPTIONS}
        value={statusTags}
        onChange={setStatusTags}
        searchable={false}
        placeholder="Alle Zustände"
        summaryThreshold={2}
      />
      <SelectedTagList
        options={STATUS_OPTIONS}
        value={statusTags}
        onRemove={(v) => setStatusTags(statusTags.filter((tag) => tag !== v))}
      />
    </fieldset>
  )
}

export default StatusFieldset
