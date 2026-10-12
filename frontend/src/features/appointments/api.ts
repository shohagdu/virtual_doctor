import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '../../api/client'
import type { Appointment, AppointmentStatus, DaySlots, VisitType } from '../../api/types'

export function useAppointments(from: string, to: string) {
  return useQuery({
    queryKey: ['appointments', from, to],
    queryFn: () => api<Appointment[]>(`/appointments?from=${from}&to=${to}`),
    placeholderData: keepPreviousData,
    refetchInterval: 30_000,
  })
}

export function usePatientAppointments(patientId: string) {
  return useQuery({
    queryKey: ['appointments', 'patient', patientId],
    queryFn: () => api<Appointment[]>(`/patients/${patientId}/appointments`),
  })
}

export function useSlots(date: string) {
  return useQuery({
    queryKey: ['slots', date],
    queryFn: () => api<DaySlots>(`/appointments/slots?date=${date}`),
    enabled: !!date,
  })
}

function useInvalidate() {
  const qc = useQueryClient()
  return () => {
    qc.invalidateQueries({ queryKey: ['appointments'] })
    qc.invalidateQueries({ queryKey: ['slots'] })
    qc.invalidateQueries({ queryKey: ['queue'] })
  }
}

export interface Booking {
  patient_id: number
  date: string
  time: string
  visit_type: VisitType
  notes: string
}

export function useBookAppointment() {
  const invalidate = useInvalidate()
  return useMutation({
    mutationFn: (b: Booking) => api<Appointment>('/appointments', { method: 'POST', body: JSON.stringify(b) }),
    onSuccess: invalidate,
  })
}

export function useUpdateAppointment() {
  const invalidate = useInvalidate()
  return useMutation({
    mutationFn: ({ id, ...patch }: { id: number; status?: AppointmentStatus; date?: string; time?: string }) =>
      api<Appointment>(`/appointments/${id}`, { method: 'PATCH', body: JSON.stringify(patch) }),
    onSettled: invalidate,
  })
}
