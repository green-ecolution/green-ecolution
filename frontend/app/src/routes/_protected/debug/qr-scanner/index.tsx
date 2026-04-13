import QRScannerView from '@/components/debug/QRScannerView'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/debug/qr-scanner/')({
  component: QRScannerView,
  loader: () => ({
    crumb: {
      title: 'QR-Scanner',
    },
  }),
})
