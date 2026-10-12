import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '../../api/client'
import { uploadFile } from '../../api/stream'
import type { AnalysisView, MedicineLine, Report, ResultInput, SafetyWarning } from '../../api/types'

export function useVisitReports(visitId: number) {
  return useQuery({
    queryKey: ['reports', 'visit', visitId],
    queryFn: () => api<Report[]>(`/visits/${visitId}/reports`),
  })
}

export function usePatientReports(patientId: string) {
  return useQuery({
    queryKey: ['reports', 'patient', patientId],
    queryFn: () => api<Report[]>(`/patients/${patientId}/reports`),
  })
}

function useInvalidateReports() {
  const qc = useQueryClient()
  return () => qc.invalidateQueries({ queryKey: ['reports'] })
}

export function useUploadReport(visitId: number) {
  const invalidate = useInvalidateReports()
  return useMutation({
    mutationFn: ({ file, name, onProgress }: { file: Blob; name: string; onProgress: (p: number) => void }) =>
      uploadFile<{ report: Report; duplicate: boolean }>(`/visits/${visitId}/reports`, file, name, onProgress),
    onSettled: invalidate,
  })
}

export function useExtractReport() {
  const invalidate = useInvalidateReports()
  return useMutation({
    mutationFn: (id: number) => api<Report>(`/reports/${id}/extract`, { method: 'POST' }),
    onSettled: invalidate,
  })
}

export interface ReportPatch {
  report_type: string | null
  lab_name: string | null
  report_date: string | null
  impression: string | null
  results: ResultInput[]
}

export function useSaveResults(id: number) {
  const invalidate = useInvalidateReports()
  return useMutation({
    mutationFn: (patch: ReportPatch) =>
      api<Report>(`/reports/${id}/results`, { method: 'PUT', body: JSON.stringify(patch) }),
    onSuccess: invalidate,
  })
}

export function useDeleteReport() {
  const invalidate = useInvalidateReports()
  return useMutation({
    mutationFn: (id: number) => api<void>(`/reports/${id}`, { method: 'DELETE' }),
    onSuccess: invalidate,
  })
}

export function useLabTrends(patientId: string, test: string | null) {
  return useQuery({
    queryKey: ['lab-trends', patientId, test],
    queryFn: () =>
      api<{
        tests: { test_code: string; test_name: string; count: number }[]
        test: string | null
        points: { report_id: number; date: string; value: number; value_text: string | null; unit: string | null; status: string | null }[]
      }>(`/patients/${patientId}/lab-trends${test ? `?test=${encodeURIComponent(test)}` : ''}`),
  })
}

export function useLatestAnalysis(visitId: number) {
  return useQuery({
    queryKey: ['analysis', visitId],
    queryFn: () => api<AnalysisView | null>(`/visits/${visitId}/ai-analysis/latest`),
  })
}

export function useSuggestionAction() {
  return useMutation({
    mutationFn: ({ id, ...body }: { id: number; action: string; edited_text?: string; reason?: string }) =>
      api<void>(`/ai-suggestions/${id}/action`, { method: 'POST', body: JSON.stringify(body) }),
  })
}

export function useSafetyCheck(visitId: number) {
  return useMutation({
    mutationFn: (medicines: MedicineLine[]) =>
      api<{ warnings: SafetyWarning[]; ai_used: boolean }>(`/visits/${visitId}/prescription/safety-check`, {
        method: 'POST',
        body: JSON.stringify({ medicines }),
      }),
  })
}
