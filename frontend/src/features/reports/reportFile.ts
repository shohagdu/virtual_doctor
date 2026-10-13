import { useQuery } from '@tanstack/react-query'
import { fileUrl } from '../../api/stream'
import type { Report } from '../../api/types'

/** The report file as a browser object URL (fetched with the login token, cached). */
export function useReportFile(r: Report, enabled = true) {
  return useQuery({
    queryKey: ['report-file', r.id],
    queryFn: () => fileUrl(`/reports/${r.id}/file`),
    enabled,
    staleTime: Infinity,
    gcTime: 30 * 60_000,
  })
}

export const isImage = (r: Report) => r.mime_type.startsWith('image/')
export const reportTitle = (r: Report, fallback: string) => r.report_type ?? r.original_name ?? fallback
