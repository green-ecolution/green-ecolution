import Option from '../Option'
import { useFilter } from '@/context/FilterContext'
import { SoilConditionOptions } from '@/hooks/details/useDetailsForSoilCondition'

const SoilFieldset = () => {
  const { filters, handleSoilChange } = useFilter()

  const groups = Array.from(new Set(SoilConditionOptions.map((o) => o.group)))

  return (
    <fieldset className="mt-6">
      <legend className="font-lato font-semibold text-dark-600 mb-2">Bodenart:</legend>
      {groups.map((group) => (
        <div key={group} className="mb-3">
          <p className="text-sm text-dark-400 mb-1">{group}</p>
          {SoilConditionOptions.filter((o) => o.group === group).map((opt) => (
            <Option
              key={opt.value}
              label={opt.label}
              name={opt.value}
              value={opt.value}
              checked={filters.soilTags.includes(opt.value)}
              onChange={handleSoilChange}
            />
          ))}
        </div>
      ))}
    </fieldset>
  )
}

export default SoilFieldset
