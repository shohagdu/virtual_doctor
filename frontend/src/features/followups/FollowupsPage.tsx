import { useState } from 'react'
import { Link, useNavigate, useSearchParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { differenceInCalendarDays, format, parseISO } from 'date-fns'
import type { Followup, FollowupBucket } from '../../api/types'
import { Card } from '../../components/Card'
import { ageText } from '../../utils/age'
import { fmtDate, todayIso } from '../../utils/date'
import { fullName } from '../../utils/patient'
import { useFollowups, useRetrySms, useUpdateFollowup } from './api'
import { SmsDialog } from './SmsDialog'

const BUCKETS: FollowupBucket[] = ['today', 'overdue', 'week', 'done']

/** Follow-ups Due (§5.7): Today / Overdue / This week, with call, SMS and came / missed. */
export function FollowupsPage() {
  const { t } = useTranslation()
  const [params, setParams] = useSearchParams()
  const bucket = (params.get('tab') as FollowupBucket) || 'today'
  const { data, isError, error, isFetching } = useFollowups(bucket)
  const [smsFor, setSmsFor] = useState<Followup | null>(null)
  const [today] = useState(todayIso)

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-2">
        <h1 className="text-2xl font-bold tracking-tight text-slate-900">{t('nav.followups')}</h1>
        <div className="flex flex-wrap gap-1" role="tablist">
          {BUCKETS.map((b) => {
            const n = b === 'done' ? null : data?.counts[b]
            return (
              <button
                key={b}
                role="tab"
                aria-selected={bucket === b}
                onClick={() => setParams({ tab: b }, { replace: true })}
                className={`rounded-md px-3 py-1.5 text-sm ${bucket === b ? 'bg-brand text-white' : 'border bg-white text-slate-600'}`}
              >
                {t(`followups.bucket.${b}`)}
                {n != null && (
                  <span
                    className={`ml-1.5 rounded-full px-1.5 text-xs ${
                      bucket === b ? 'bg-white/25' : b === 'overdue' && n > 0 ? 'bg-red-100 text-red-700' : 'bg-slate-100'
                    }`}
                  >
                    {n}
                  </span>
                )}
              </button>
            )
          })}
        </div>
      </div>

      <Card>
        {isError ? (
          <p className="text-red-600">{error.message}</p>
        ) : !data ? (
          <p className="text-slate-500">{t('common.loading')}</p>
        ) : data.items.length === 0 ? (
          <p className="text-slate-500">{t(`followups.empty.${bucket}`)}</p>
        ) : (
          <div className={`overflow-x-auto ${isFetching ? 'opacity-70' : ''}`}>
            <table className="w-full text-left text-sm">
              <thead className="border-b text-slate-500">
                <tr>
                  <th className="py-2 pr-3">{t('followups.due')}</th>
                  <th className="py-2 pr-3">{t('patients.name')}</th>
                  <th className="py-2 pr-3">{t('patients.phone')}</th>
                  <th className="py-2 pr-3">{t('followups.lastVisit')}</th>
                  <th className="py-2 pr-3">{t('followups.sms')}</th>
                  <th className="py-2 pr-3">{t('queue.action')}</th>
                </tr>
              </thead>
              <tbody>
                {data.items.map((f) => (
                  <Row key={f.id} f={f} today={today} onSms={() => setSmsFor(f)} />
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>
      <p className="text-xs text-slate-500">{t('followups.autoCame')}</p>

      {smsFor && <SmsDialog followup={smsFor} onClose={() => setSmsFor(null)} />}
    </div>
  )
}

function Row({ f, today, onSms }: { f: Followup; today: string; onSms: () => void }) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const update = useUpdateFollowup()
  const retry = useRetrySms()
  const overdueDays = differenceInCalendarDays(parseISO(today), parseISO(f.due_date))
  const pending = f.status === 'pending'
  // Overdue chronic patients (diabetes, hypertension …) stand out (§5.7).
  const chronicOverdue = pending && overdueDays > 0 && f.chronic_conditions.length > 0

  return (
    <tr className={`border-b align-top last:border-0 ${chronicOverdue ? 'bg-red-50' : ''}`}>
      <td className="py-2 pr-3 whitespace-nowrap">
        {fmtDate(f.due_date)}
        {pending && overdueDays > 0 && (
          <div className="text-xs font-medium text-red-700">{t('followups.daysOverdue', { count: overdueDays })}</div>
        )}
        {!pending && (
          <div className={`text-xs ${f.status === 'came' ? 'text-green-700' : 'text-red-700'}`}>
            {f.status === 'came' ? '✔ ' : '✕ '}
            {t(`followups.status.${f.status}`)}
          </div>
        )}
      </td>
      <td className="py-2 pr-3">
        <Link to={`/patients/${f.patient.id}`} className="font-medium text-brand hover:underline">
          {fullName(f.patient)}
        </Link>
        <div className="text-xs text-slate-500">
          {f.patient.patient_code} · {ageText(f.patient.dob)} / {f.patient.sex}
        </div>
        {f.chronic_conditions.length > 0 && (
          <div className="mt-0.5 flex flex-wrap gap-1">
            {f.chronic_conditions.map((c) => (
              <span key={c} className={`rounded-md px-1.5 text-xs ${chronicOverdue ? 'bg-red-600 text-white' : 'bg-slate-200'}`}>
                {c}
              </span>
            ))}
          </div>
        )}
      </td>
      <td className="py-2 pr-3 whitespace-nowrap">
        <a href={`tel:${f.patient.phone}`} className="text-brand hover:underline">
          📞 {f.patient.phone}
        </a>
      </td>
      <td className="py-2 pr-3">
        {f.visit_date ? fmtDate(f.visit_date) : '—'}
        {f.diagnosis && f.diagnosis.length > 0 && <div className="text-xs text-slate-500">{f.diagnosis.join(', ')}</div>}
      </td>
      <td className="py-2 pr-3 text-xs">
        <SmsStatus f={f} onRetry={(id) => retry.mutate(id)} retrying={retry.isPending} />
      </td>
      <td className="py-2 pr-3">
        <div className="flex flex-wrap gap-1">
          {pending ? (
            <>
              <button onClick={onSms} className="rounded-md border px-2 py-1 text-xs">
                💬 SMS
              </button>
              <button
                onClick={() => navigate(`/appointments?book=${f.patient.id}`)}
                className="rounded-md border px-2 py-1 text-xs"
              >
                📅 {t('appointments.book')}
              </button>
              <button
                onClick={() => update.mutate({ id: f.id, status: 'came' })}
                disabled={update.isPending}
                className="rounded-md border border-green-600 px-2 py-1 text-xs text-green-700 disabled:opacity-50"
              >
                ✔ {t('followups.status.came')}
              </button>
              <button
                onClick={() => update.mutate({ id: f.id, status: 'missed' })}
                disabled={update.isPending}
                className="rounded-md border border-red-600 px-2 py-1 text-xs text-red-700 disabled:opacity-50"
              >
                ✕ {t('followups.status.missed')}
              </button>
            </>
          ) : (
            <button
              onClick={() => update.mutate({ id: f.id, status: 'pending' })}
              disabled={update.isPending}
              className="rounded-md border px-2 py-1 text-xs"
            >
              ↺ {t('followups.reopen')}
            </button>
          )}
        </div>
        {update.isError && <p className="text-xs text-red-600">{update.error.message}</p>}
      </td>
    </tr>
  )
}

/** Latest reminder: sending / sent / failed with retry (§13 "SMS failed"). */
function SmsStatus({ f, onRetry, retrying }: { f: Followup; onRetry: (id: number) => void; retrying: boolean }) {
  const { t } = useTranslation()
  const s = f.last_sms
  if (!s) return <span className="text-slate-400">—</span>
  const when = format(new Date(s.created_at), 'dd MMM, h:mm a')
  if (s.status === 'sent') return <span className="text-green-700" title={when}>✔ {t('followups.smsSent')} <span className="text-slate-500">{when}</span></span>
  if (s.status === 'queued')
    return (
      <span className="text-slate-600" title={s.error ?? undefined}>
        ⏳ {s.error ? t('followups.smsRetrying') : t('followups.smsQueued')}
      </span>
    )
  return (
    <span className="text-red-700">
      ⚠ {t('followups.smsFailed')}
      {s.error && <span className="block text-slate-500">{s.error}</span>}
      <button onClick={() => onRetry(s.id)} disabled={retrying} className="mt-0.5 rounded-md border border-red-600 px-1.5 disabled:opacity-50">
        ↻ {t('followups.retry')}
      </button>
    </span>
  )
}
