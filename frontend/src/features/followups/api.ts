import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api } from '../../api/client'
import type { FollowupBucket, FollowupList, FollowupStatus, SmsDraft } from '../../api/types'

export function useFollowups(bucket: FollowupBucket) {
  return useQuery({
    queryKey: ['followups', bucket],
    queryFn: () => api<FollowupList>(`/followups?bucket=${bucket}`),
    placeholderData: keepPreviousData,
    // Picks up SMS delivery results from the background sender.
    refetchInterval: 20_000,
  })
}

function useInvalidate() {
  const qc = useQueryClient()
  return () => {
    qc.invalidateQueries({ queryKey: ['followups'] })
    qc.invalidateQueries({ queryKey: ['queue'] })
  }
}

export function useUpdateFollowup() {
  const invalidate = useInvalidate()
  return useMutation({
    mutationFn: ({ id, status }: { id: number; status: FollowupStatus }) =>
      api<void>(`/followups/${id}`, { method: 'PATCH', body: JSON.stringify({ status }) }),
    onSettled: invalidate,
  })
}

export function useSmsDraft(followupId: number) {
  return useQuery({
    queryKey: ['followups', 'sms', followupId],
    queryFn: () => api<SmsDraft>(`/followups/${followupId}/sms`),
    staleTime: 0,
    gcTime: 0,
  })
}

export function useSendSms() {
  const invalidate = useInvalidate()
  return useMutation({
    mutationFn: ({ id, body }: { id: number; body: string }) =>
      api<void>(`/followups/${id}/sms`, { method: 'POST', body: JSON.stringify({ body }) }),
    onSuccess: invalidate,
  })
}

export function useRetrySms() {
  const invalidate = useInvalidate()
  return useMutation({
    mutationFn: (smsId: number) => api<void>(`/sms/${smsId}/retry`, { method: 'POST' }),
    onSettled: invalidate,
  })
}
