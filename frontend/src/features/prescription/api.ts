import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '../../api/client'
import type {
  AllergyMatch,
  MedicineLine,
  MedicineSuggestion,
  Prescription,
  PrescriptionBody,
  VisitContext,
} from '../../api/types'
import { emptyLine } from './medicine'

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
    mutationFn: ({ base, body, allergy_ack }: { base: Base; body: PrescriptionBody; allergy_ack: boolean }) =>
      api<Prescription>(`/visits/${visitId}/prescription`, {
        method: 'PUT',
        body: JSON.stringify({ base, allergy_ack, ...body }),
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

/** Live allergy warnings (PM-09); the server runs the same check on save. */
export function useAllergyCheck(visitId: string, medicines: MedicineLine[], enabled: boolean) {
  const lines = medicines
    .filter((m) => m.brand.trim())
    .map((m) => ({ ...emptyLine, brand: m.brand, generic: m.generic, medicine_id: m.medicine_id }))
  return useQuery({
    queryKey: ['allergy-check', visitId, lines.map((l) => [l.brand, l.generic])],
    queryFn: () =>
      api<AllergyMatch[]>(`/visits/${visitId}/allergy-check`, {
        method: 'POST',
        body: JSON.stringify({ medicines: lines }),
      }),
    enabled: enabled && lines.length > 0,
    placeholderData: keepPreviousData,
    staleTime: 60_000,
  })
}
