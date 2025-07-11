import { Tree } from '@green-ecolution/backend-client'
import { format } from 'date-fns'
import DetailedList from '../general/DetailedList'

interface TabGeneralDataProps {
  tree?: Tree
}

const TabGeneralData: React.FC<TabGeneralDataProps> = ({ tree }) => {
  const updatedDate = tree?.updatedAt
    ? format(new Date(tree?.updatedAt), 'dd.MM.yyyy')
    : 'Keine Angabe'

  const treeData = [
    {
      label: 'Baumart',
      value: tree?.species ?? 'Keine Angabe',
    },
    {
      label: 'Pflanzjahr',
      value: `${tree?.plantingYear ?? 'Keine Angabe'}`,
    },
    {
      label: 'Ursprung der Daten',
      value: `${tree?.provider ?? 'Manuell erstellter Baum'}`,
    },
    {
      label: 'Datum der letzten Bewässerung',
      value: tree?.lastWatered ? format(new Date(tree.lastWatered), 'dd.MM.yyyy') : 'Keine Angabe',
    },
    {
      label: 'Latitude',
      value: `${tree?.latitude ?? 'Keine Angabe'}`,
    },
    {
      label: 'Longitude',
      value: `${tree?.longitude ?? 'Keine Angabe'}`,
    },
    {
      label: 'Letztes Update',
      value: updatedDate,
    },
  ]

  return <DetailedList details={treeData} />
}

export default TabGeneralData
