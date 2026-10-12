import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '../../api/client'
import type { Vitals, VitalsRecord } from '../../api/types'

export function useVitalsTrend(patientId: string) {
  return useQuery({
    queryKey: ['vitals-trend', patientId],
    queryFn: () => api<VitalsRecord[]>(`/patients/${patientId}/vitals/trend`),
  })
}

export function useVisitVitals(visitId: number) {
  return useQuery({
    queryKey: ['vitals', visitId],
    queryFn: () => api<VitalsRecord | null>(`/visits/${visitId}/vitals`),
  })
}

export function useSaveVitals(visitId: number, patientId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: Vitals) => api<VitalsRecord>(`/visits/${visitId}/vitals`, { method: 'PUT', body: JSON.stringify(v) }),
    onSuccess: (rec) => {
      qc.setQueryData(['vitals', visitId], rec)
      qc.invalidateQueries({ queryKey: ['vitals-trend', String(patientId)] })
      qc.invalidateQueries({ queryKey: ['visit', String(visitId)] })
      qc.invalidateQueries({ queryKey: ['queue'] })
    },
  })
}
