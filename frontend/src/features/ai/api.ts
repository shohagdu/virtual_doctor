import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '../../api/client'
import type { AiUsageRow, Patient, SavedSummary, SearchFilters } from '../../api/types'

/** Whether AI can be offered: switched on for the clinic and set up on the server. */
export function useAiStatus() {
  return useQuery({
    queryKey: ['ai-status'],
    queryFn: () => api<{ configured: boolean; enabled: boolean }>('/ai/status'),
    staleTime: 5 * 60_000,
  })
}

export function useAiAvailable() {
  const { data } = useAiStatus()
  return !!data?.configured && !!data.enabled
}

export function useLatestSummary(patientId: string, enabled = true) {
  return useQuery({
    queryKey: ['ai-summary', patientId],
    queryFn: () => api<SavedSummary | null>(`/patients/${patientId}/ai-summary/latest`),
    enabled,
  })
}

export function useSmartSearch(q: string) {
  return useQuery({
    queryKey: ['smart-search', q],
    queryFn: () => api<{ filters: SearchFilters; items: Patient[] }>(`/patients/smart-search?q=${encodeURIComponent(q)}`),
    enabled: q.trim().length >= 3,
    staleTime: 60_000,
    retry: false,
  })
}

export function useAiUsage(from: string, to: string) {
  return useQuery({
    queryKey: ['ai-usage', from, to],
    queryFn: () => api<{ rows: AiUsageRow[] }>(`/admin/ai-usage?from=${from}&to=${to}`),
  })
}

export function useSmsAiDraft() {
  return useMutation({
    mutationFn: (followupId: number) =>
      api<{ body: string; segments: number }>(`/followups/${followupId}/sms/ai-draft`, { method: 'POST' }),
  })
}

export function useInvalidateSummary(patientId: string) {
  const qc = useQueryClient()
  return () => qc.invalidateQueries({ queryKey: ['ai-summary', patientId] })
}
