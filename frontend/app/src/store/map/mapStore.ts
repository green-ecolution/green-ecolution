import { SubStore } from '../store'
import { MapStore } from './types'

export const mapStore: SubStore<MapStore> = (set) => ({
  center: [54.792277136221905, 9.43580607453268],
  zoom: 13,
  minZoom: 13,
  maxZoom: 18,
  showSelectModal: false,
  setCenter: (center) =>
    set((state) => {
      state.map.center = center
    }),
  setZoom: (zoom) =>
    set((state) => {
      state.map.zoom = zoom
    }),
  setShowSelectModal: (showSelectModal) =>
    set((state) => {
      state.map.showSelectModal = showSelectModal
    }),
})
