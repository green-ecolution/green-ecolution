import { useMutation } from '@tanstack/react-query'
import { basePath } from '@/api/backendApi'
import { useAuthSession } from '@/lib/auth/authSessionContext'
import createToast from '@/hooks/createToast'
import { isHTTPError } from '@/lib/utils'

/** Downloads the GPX file behind `gpxUrl` and triggers a browser save dialog. */
export const useDownloadGpx = (gpxUrl: string) => {
  const { accessToken } = useAuthSession()
  const showToast = createToast()

  return useMutation({
    mutationFn: async () => {
      const resp = await fetch(`${basePath}${gpxUrl}`, {
        method: 'GET',
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      if (resp.status !== 200) {
        const json: unknown = await resp.json()
        const errorMsg = isHTTPError(json) ? json.error : 'Unbekannter Fehler'
        throw new Error(errorMsg)
      }

      const blob = await resp.blob()

      const objUrl = window.URL.createObjectURL(blob)

      const a = document.createElement('a')
      a.href = objUrl
      a.download = resp.headers.get('Content-Disposition')?.split('filename=')[1] ?? 'route.gpx'
      a.click()

      window.URL.revokeObjectURL(objUrl)
    },
    onError: (error) => {
      showToast(error.message, 'error')
    },
  })
}
