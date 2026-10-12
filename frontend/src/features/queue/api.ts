import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '../../api/client'
import type { QueueToday, Visit, VisitStatus, VisitType } from '../../api/types'

export function useQueueToday() {
  return useQuery({
    queryKey: ['queue'],
    queryFn: () => api<QueueToday>('/queue/today'),
    // Several people work the same queue (doctor, assistant at the door).
    refetchInterval: 15_000,
  })
}

function useInvalidateQueue() {
  const qc = useQueryClient()
  return () => {
    qc.invalidateQueries({ queryKey: ['queue'] })
    qc.invalidateQueries({ queryKey: ['visits'] })
  }
}

export function useAddToQueue() {
  const invalidate = useInvalidateQueue()
  return useMutation({
    mutationFn: ({ patient_id, visit_type }: { patient_id: number; visit_type: VisitType }) =>
      api<Visit>('/queue/today', { method: 'POST', body: JSON.stringify({ patient_id, visit_type }) }),
    onSuccess: invalidate,
  })
}

export function useUpdateQueue() {
  const invalidate = useInvalidateQueue()
  return useMutation({
    mutationFn: ({ id, ...patch }: { id: number; status?: VisitStatus; position?: number }) =>
      api<Visit>(`/queue/${id}`, { method: 'PATCH', body: JSON.stringify(patch) }),
    onSettled: invalidate,
  })
}

/** Save & Next Patient: marks the current visit seen and calls the next waiting one. */
export function useCallNext() {
  const invalidate = useInvalidateQueue()
  return useMutation({
    mutationFn: (current_visit_id: number | null) =>
      api<{ next: Visit | null }>('/queue/next', { method: 'POST', body: JSON.stringify({ current_visit_id }) }),
    onSuccess: invalidate,
  })
}
