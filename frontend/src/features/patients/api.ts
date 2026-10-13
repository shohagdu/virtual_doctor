import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '../../api/client'
import type {
  CreatePatientResponse,
  MedicalProfile,
  Page,
  Patient,
  PatientListItem,
  PatientSort,
  Visit,
  VisitSummary,
  VisitType,
} from '../../api/types'
import type { PatientForm } from './schema'

export function usePatients(
  q: string,
  page: number,
  enabled = true,
  opts: { sex?: string; sort?: PatientSort; perPage?: number } = {},
) {
  const qs = new URLSearchParams({ q, page: String(page), per_page: String(opts.perPage ?? 20) })
  if (opts.sex) qs.set('sex', opts.sex)
  if (opts.sort) qs.set('sort', opts.sort)
  return useQuery({
    queryKey: ['patients', q, page, opts.sex ?? '', opts.sort ?? '', opts.perPage ?? 20],
    queryFn: () => api<Page<PatientListItem>>(`/patients?${qs}`),
    placeholderData: keepPreviousData,
    enabled,
  })
}

export function usePatient(id: string) {
  return useQuery({
    queryKey: ['patient', id],
    queryFn: () => api<Patient>(`/patients/${id}`),
    enabled: !!id,
  })
}

export function useCreatePatient() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ form, force }: { form: PatientForm; force: boolean }) =>
      api<CreatePatientResponse>(`/patients${force ? '?force=true' : ''}`, {
        method: 'POST',
        body: JSON.stringify(form),
      }),
    onSuccess: (res) => {
      if (res.patient) qc.invalidateQueries({ queryKey: ['patients'] })
    },
  })
}

export function useUpdatePatient(id: number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (form: PatientForm) =>
      api<Patient>(`/patients/${id}`, { method: 'PATCH', body: JSON.stringify(form) }),
    onSuccess: (p) => {
      qc.setQueryData(['patient', String(id)], p)
      qc.invalidateQueries({ queryKey: ['patients'] })
    },
  })
}

export function useMedicalProfile(id: string, enabled = true) {
  return useQuery({
    queryKey: ['medical-profile', id],
    queryFn: () => api<MedicalProfile>(`/patients/${id}/medical-profile`),
    enabled,
  })
}

export function useSaveMedicalProfile(id: string) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (profile: MedicalProfile) =>
      api<MedicalProfile>(`/patients/${id}/medical-profile`, { method: 'PUT', body: JSON.stringify(profile) }),
    onSuccess: (p) => qc.setQueryData(['medical-profile', id], p),
  })
}

export function useVisits(patientId: string, enabled = true) {
  return useQuery({
    queryKey: ['visits', patientId],
    queryFn: () => api<VisitSummary[]>(`/patients/${patientId}/visits`),
    enabled,
  })
}

export function useStartVisit(patientId: string | number) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (visit_type: VisitType) =>
      api<Visit>(`/patients/${patientId}/visits`, { method: 'POST', body: JSON.stringify({ visit_type }) }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['visits', String(patientId)] })
      qc.invalidateQueries({ queryKey: ['queue'] })
    },
  })
}
