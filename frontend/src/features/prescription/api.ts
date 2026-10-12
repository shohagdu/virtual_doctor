import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '../../api/client'
import type { MedicineSuggestion, Prescription, PrescriptionBody, VisitContext } from '../../api/types'

type Base = { id: number; revision: number } | null

export function useVisitContext(visitId: string) {
  return useQuery({ queryKey: ['visit', visitId], queryFn: () => api<VisitContext>(`/visits/${visitId}`) })
}

export function usePrescription(visitId: string) {
  return useQuery({
    queryKey: ['prescription', visitId],
    queryFn: () => api<Prescription>(`/visits/${visitId}/prescription`),
    // Never refetch behind the doctor's back while they are editing.
    staleTime: Infinity,
    refetchOnWindowFocus: false,
  })
}

export function useSavePrescription(visitId: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ base, body }: { base: Base; body: PrescriptionBody }) =>
      api<Prescription>(`/visits/${visitId}/prescription`, {
        method: 'PUT',
        body: JSON.stringify({ base, ...body }),
      }),
    onSuccess: (rx) => {
      qc.setQueryData(['prescription', visitId], rx)
      qc.invalidateQueries({ queryKey: ['visits'] })
    },
  })
}

export function useLockPrescription(visitId: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (base: { id: number; revision: number }) =>
      api<Prescription>(`/visits/${visitId}/prescription/lock`, { method: 'POST', body: JSON.stringify(base) }),
    onSuccess: (rx) => {
      qc.setQueryData(['prescription', visitId], rx)
      qc.invalidateQueries({ queryKey: ['visits'] })
      qc.invalidateQueries({ queryKey: ['visit', visitId] })
    },
  })
}

export function useMedicineSearch(q: string) {
  return useQuery({
    queryKey: ['medicines', q],
    queryFn: () => api<MedicineSuggestion[]>(`/medicines/search?q=${encodeURIComponent(q)}`),
    enabled: q.trim().length >= 2,
    staleTime: 60_000,
  })
}
