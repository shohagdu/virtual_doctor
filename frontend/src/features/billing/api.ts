import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '../../api/client'
import type { Bill, BillInput, Collection } from '../../api/types'

export function useVisitBill(visitId: number) {
  return useQuery({
    queryKey: ['bill', visitId],
    queryFn: () => api<{ bill: Bill | null; suggested_fee: number }>(`/visits/${visitId}/bill`),
  })
}

export function useSaveBill(visitId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (b: BillInput) => api<Bill>(`/visits/${visitId}/bill`, { method: 'PUT', body: JSON.stringify(b) }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['bill', visitId] })
      qc.invalidateQueries({ queryKey: ['queue'] })
      qc.invalidateQueries({ queryKey: ['collection'] })
    },
  })
}

export function useCollection(from: string, to: string) {
  return useQuery({
    queryKey: ['collection', from, to],
    queryFn: () => api<Collection>(`/reports/collection?from=${from}&to=${to}`),
  })
}
