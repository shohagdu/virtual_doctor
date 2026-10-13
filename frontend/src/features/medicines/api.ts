import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '../../api/client'
import type { Medicine, MedicineInput } from '../../api/types'

export interface MedicineQuery {
  q: string
  form: string
  company: string
  status: '' | 'active' | 'inactive'
  page: number
}

export function useMedicines(f: MedicineQuery) {
  const qs = new URLSearchParams({ q: f.q, form: f.form, company: f.company, status: f.status, page: String(f.page), per_page: '25' })
  return useQuery({
    queryKey: ['medicines', f],
    queryFn: () =>
      api<{ items: Medicine[]; total: number; page: number; per_page: number; forms: string[]; companies: string[] }>(`/medicines?${qs}`),
    placeholderData: keepPreviousData,
  })
}

function useInvalidate() {
  const qc = useQueryClient()
  return () => {
    qc.invalidateQueries({ queryKey: ['medicines'] })
    qc.invalidateQueries({ queryKey: ['catalogue'] })
  }
}

export function useSaveMedicine() {
  const invalidate = useInvalidate()
  return useMutation({
    mutationFn: ({ id, ...m }: MedicineInput & { id?: number }) =>
      api<Medicine>(id ? `/medicines/${id}` : '/medicines', { method: id ? 'PUT' : 'POST', body: JSON.stringify(m) }),
    onSuccess: invalidate,
  })
}

export function useSetActive() {
  const invalidate = useInvalidate()
  return useMutation({
    mutationFn: ({ id, is_active }: { id: number; is_active: boolean }) =>
      api<Medicine>(`/medicines/${id}`, { method: 'PATCH', body: JSON.stringify({ is_active }) }),
    onSettled: invalidate,
  })
}

export function useDeleteMedicine() {
  const invalidate = useInvalidate()
  return useMutation({
    mutationFn: (id: number) => api<void>(`/medicines/${id}`, { method: 'DELETE' }),
    onSettled: invalidate,
  })
}
