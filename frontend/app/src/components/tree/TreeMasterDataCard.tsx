import { Card, CardContent, CardHeader, CardTitle, DetailedList } from '@green-ecolution/ui'
import { format } from 'date-fns'
import type { Tree } from '@/api/backendApi'

interface TreeMasterDataCardProps {
  tree: Tree
}

const TreeMasterDataCard = ({ tree }: TreeMasterDataCardProps) => {
  const details = [
    { label: 'Baumart', value: tree.species || 'Keine Angabe' },
    { label: 'Baumnummer', value: tree.number || 'Keine Angabe' },
    { label: 'Pflanzjahr', value: `${tree.plantingYear}` },
    { label: 'Ursprung der Daten', value: tree.provider ?? 'Manuell erstellt' },
    { label: 'Koordinaten', value: `${tree.latitude.toFixed(6)}, ${tree.longitude.toFixed(6)}` },
    { label: 'Letztes Update', value: format(new Date(tree.updatedAt), 'dd.MM.yyyy') },
  ]

  return (
    <Card variant="outlined">
      <CardHeader>
        <CardTitle>Stammdaten</CardTitle>
      </CardHeader>
      <CardContent>
        <DetailedList details={details} columns={1} />
      </CardContent>
    </Card>
  )
}

export default TreeMasterDataCard
