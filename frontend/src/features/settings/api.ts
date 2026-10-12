import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '../../api/client'
import type { ClinicSettings } from '../../api/types'

/** Fees, chamber schedule and SMS settings (§12). */
export function useClinicSettings() {
  return useQuery({
    queryKey: ['settings'],
    queryFn: () => api<ClinicSettings>('/settings'),
    staleTime: 5 * 60_000,
  })
}

export function useSaveClinicSettings() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (s: ClinicSettings) => api<ClinicSettings>('/settings', { method: 'PUT', body: JSON.stringify(s) }),
    onSuccess: (s) => {
      qc.setQueryData(['settings'], s)
      qc.invalidateQueries({ queryKey: ['slots'] })
    },
  })
}
