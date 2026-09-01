import { useMutation } from '@tanstack/react-query'
import { basePath } from '@/api/backendApi'
import { useAuthSession } from '@/lib/auth/authSessionContext'
import createToast from '@/hooks/createToast'
import { resolveApiError } from '@/lib/apiError'
import { ResponseError } from '@green-ecolution/backend-client'
import { useTranslation } from 'react-i18next'

/** Downloads the GPX file behind `gpxUrl` and triggers a browser save dialog. */
export const useDownloadGpx = (gpxUrl: string) => {
  const { accessToken } = useAuthSession()
  const showToast = createToast()
  const { t } = useTranslation('errors')

  return useMutation({
    mutationFn: async () => {
      const resp = await fetch(`${basePath}${gpxUrl}`, {
        method: 'GET',
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      if (resp.status !== 200) {
        const { message } = await resolveApiError(new ResponseError(resp))
        throw new Error(message)
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
      void resolveApiError(error).then((info) =>
        showToast(t('frame.gpxDownloadFailed', { reason: info.message }), 'error'),
      )
    },
  })
}
