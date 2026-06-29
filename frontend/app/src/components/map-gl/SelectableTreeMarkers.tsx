import { useEffect, useRef } from 'react'
import { Marker } from 'maplibre-gl'
import { WateringStatus } from '@green-ecolution/backend-client'
import type { SensorResponse } from '@green-ecolution/backend-client'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { useMaplibreMap } from '@/components/map-gl/MapContext'

export interface SelectableTree {
  id: string
  latitude: number
  longitude: number
  wateringStatus?: WateringStatus
  sensor?: SensorResponse | null
}

interface SelectableTreeMarkersProps {
  trees: SelectableTree[]
  selectedTreeId: string | null
  onSelect?: (treeId: string) => void
}

const buildTreeElement = (colorHex: string, isSelected: boolean, isAssigned: boolean) => {
  const el = document.createElement('div')
  el.style.width = '20px'
  el.style.height = '20px'
  el.style.borderRadius = '9999px'
  el.style.background = colorHex
  el.style.border = '2px solid #ffffff'
  el.style.boxShadow = isSelected ? '0 0 0 3px #486725' : '0 1px 3px rgba(0,0,0,0.4)'
  el.style.opacity = isAssigned ? '0.45' : '1'
  el.style.cursor = isAssigned ? 'default' : 'pointer'
  return el
}

const SelectableTreeMarkers = ({ trees, selectedTreeId, onSelect }: SelectableTreeMarkersProps) => {
  const map = useMaplibreMap()
  const markersRef = useRef<Marker[]>([])
  const onSelectRef = useRef(onSelect)
  useEffect(() => {
    onSelectRef.current = onSelect
  })

  useEffect(() => {
    for (const m of markersRef.current) m.remove()
    markersRef.current = trees.map((tree) => {
      const { colorHex } = getWateringStatusDetails(tree.wateringStatus ?? WateringStatus.Unknown)
      const isAssigned = tree.sensor != null
      const isSelected = !isAssigned && tree.id === selectedTreeId
      const el = buildTreeElement(colorHex, isSelected, isAssigned)
      if (!isAssigned) {
        el.addEventListener('click', () => onSelectRef.current?.(tree.id))
      }
      return new Marker({ element: el }).setLngLat([tree.longitude, tree.latitude]).addTo(map)
    })
    return () => {
      for (const m of markersRef.current) m.remove()
      markersRef.current = []
    }
  }, [map, trees, selectedTreeId])

  return null
}

export default SelectableTreeMarkers
