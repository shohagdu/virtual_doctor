import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '../../api/client'
import type { CreatePatientResponse, Page, Patient } from '../../api/types'
import type { PatientForm } from './schema'

export function usePatients(q: string, page: number) {
  return useQuery({
    queryKey: ['patients', q, page],
    queryFn: () => api<Page<Patient>>(`/patients?q=${encodeURIComponent(q)}&page=${page}`),
    placeholderData: keepPreviousData,
  })
}

export function usePatient(id: string) {
  return useQuery({
    queryKey: ['patient', id],
    queryFn: () => api<Patient>(`/patients/${id}`),
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
