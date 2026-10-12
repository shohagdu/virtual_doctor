import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '../../api/client'
import type {
  ClinicDocument,
  CollectionRow,
  DocumentBody,
  DocumentKind,
  MedicineFavorite,
  MedicineLine,
  MergeSide,
  PrescriptionBody,
  RxTemplate,
} from '../../api/types'

export function useTemplates() {
  return useQuery({ queryKey: ['rx-templates'], queryFn: () => api<RxTemplate[]>('/rx-templates') })
}

export function useSaveTemplate() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (t: { name: string; body: PrescriptionBody }) =>
      api<RxTemplate[]>('/rx-templates', { method: 'POST', body: JSON.stringify(t) }),
    onSuccess: (list) => qc.setQueryData(['rx-templates'], list),
  })
}

export function useDeleteTemplate() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: number) => api<void>(`/rx-templates/${id}`, { method: 'DELETE' }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['rx-templates'] }),
  })
}

export function useFavorites() {
  return useQuery({ queryKey: ['favorites'], queryFn: () => api<MedicineFavorite[]>('/medicine-favorites') })
}

export function useAddFavorite() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (line: MedicineLine) =>
      api<MedicineFavorite[]>('/medicine-favorites', { method: 'POST', body: JSON.stringify(line) }),
    onSuccess: (list) => qc.setQueryData(['favorites'], list),
  })
}

export function useDeleteFavorite() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: number) => api<void>(`/medicine-favorites/${id}`, { method: 'DELETE' }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['favorites'] }),
  })
}

export function useDocuments(visitId: number) {
  return useQuery({
    queryKey: ['documents', visitId],
    queryFn: () => api<ClinicDocument[]>(`/visits/${visitId}/documents`),
  })
}

export function useCreateDocument(visitId: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (d: { kind: DocumentKind; body: DocumentBody }) =>
      api<ClinicDocument>(`/visits/${visitId}/documents`, { method: 'POST', body: JSON.stringify(d) }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['documents', visitId] }),
  })
}

export function usePatientBills(patientId: string) {
  return useQuery({ queryKey: ['bills', patientId], queryFn: () => api<CollectionRow[]>(`/patients/${patientId}/bills`) })
}

export function useMergePreview(keep: number, merge: number | null) {
  return useQuery({
    queryKey: ['merge-preview', keep, merge],
    queryFn: () => api<{ keep: MergeSide; merge: MergeSide }>(`/admin/patients/merge-preview?keep=${keep}&merge=${merge}`),
    enabled: merge !== null,
  })
}

export function useMergePatients() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (p: { keep: number; merge: number }) =>
      api<void>('/admin/patients/merge', { method: 'POST', body: JSON.stringify(p) }),
    onSuccess: () => qc.invalidateQueries(),
  })
}

export function useCatalogue() {
  return useQuery({
    queryKey: ['catalogue'],
    queryFn: () => api<{ clinic_rows: number; shared_rows: number }>('/medicines/catalogue'),
  })
}

export function useImportCatalogue() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (csv: string) =>
      api<{ added: number; updated: number; problems: { line: number; message: string }[]; more_problems: number }>(
        '/medicines/import',
        { method: 'POST', body: csv, headers: { 'Content-Type': 'text/csv' } },
      ),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['catalogue'] }),
  })
}
