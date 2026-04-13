import type { BarcodeDetector as BarcodeDetectorType } from 'barcode-detector/pure'

declare global {
  interface Window {
    BarcodeDetector?: typeof BarcodeDetectorType
  }
}

export {}
