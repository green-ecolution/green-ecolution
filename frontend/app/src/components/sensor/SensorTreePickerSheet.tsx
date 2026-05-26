import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@green-ecolution/ui'
import { useState } from 'react'
import SensorTreeSearchInput from './SensorTreeSearchInput'
import SensorTreeSearchResults from './SensorTreeSearchResults'

interface SensorTreePickerSheetProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  selectedTreeId: string | null
  onSelect: (treeId: string) => void
}

interface PickerBodyProps {
  selectedTreeId: string | null
  onSelect: (treeId: string) => void
}

const PickerBody = ({ selectedTreeId, onSelect }: PickerBodyProps) => {
  const [q, setQ] = useState('')

  return (
    <>
      <div className="px-6 pb-3">
        <SensorTreeSearchInput value={q} onChange={setQ} />
      </div>
      <div className="flex-1 overflow-y-auto px-6 pb-6">
        <SensorTreeSearchResults q={q} selectedTreeId={selectedTreeId} onSelect={onSelect} />
      </div>
    </>
  )
}

const SensorTreePickerSheet = ({
  open,
  onOpenChange,
  selectedTreeId,
  onSelect,
}: SensorTreePickerSheetProps) => {
  const handleSelect = (treeId: string) => {
    onSelect(treeId)
    onOpenChange(false)
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md sm:max-w-lg h-[85vh] sm:h-[70vh] flex flex-col gap-0 p-0">
        <DialogHeader className="px-6 pt-6 pb-3">
          <DialogTitle>Anderen Baum auswählen</DialogTitle>
          <DialogDescription>
            Suche nach Baumnummer oder Baumart, wenn der richtige Baum nicht in der Vorschlagsliste
            ist.
          </DialogDescription>
        </DialogHeader>
        <PickerBody
          key={open ? 'open' : 'closed'}
          selectedTreeId={selectedTreeId}
          onSelect={handleSelect}
        />
      </DialogContent>
    </Dialog>
  )
}

export default SensorTreePickerSheet
export type { SensorTreePickerSheetProps }
