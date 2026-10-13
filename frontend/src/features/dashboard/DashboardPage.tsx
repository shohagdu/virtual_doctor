import { useDeferredValue, useState, type DragEvent } from 'react'
import { Link, useNavigate } from 'react-router'
import { useTranslation } from 'react-i18next'
import type { QueueEntry, VisitType } from '../../api/types'
import { AlertTriangle, BellRing, CheckCircle2, Coffee, GripVertical, Hourglass, ListOrdered, UserPlus, Users } from 'lucide-react'
import { Card } from '../../components/Card'
import { PageHeader } from '../../components/PageHeader'
import { StatCard } from '../../components/StatCard'
import { StatusBadge } from '../../components/StatusBadge'
import { ageText } from '../../utils/age'
import { fullName } from '../../utils/patient'
import { useAuth } from '../auth/authStore'
import { BillDialog } from '../billing/BillDialog'
import { usePatients } from '../patients/api'
import { PatientFormDialog } from '../patients/PatientFormDialog'
import { useAddToQueue, useQueueToday, useUpdateQueue } from '../queue/api'
import { VitalsDialog } from '../vitals/VitalsDialog'

export function DashboardPage() {
  const { t } = useTranslation()
  const user = useAuth((s) => s.user)
  const { data, isError, error } = useQueueToday()
  const [showNew, setShowNew] = useState(false)
  const [hour] = useState(() => new Date().getHours())
  const c = data?.counters
  const greeting = hour < 12 ? 'dashboard.morning' : hour < 17 ? 'dashboard.afternoon' : 'dashboard.evening'
  const done = c && c.today > 0 ? Math.round((c.seen / c.today) * 100) : 0

  return (
    <div className="space-y-6">
      <PageHeader
        title={t(greeting, { name: user?.name ?? '' })}
        subtitle={c ? t('dashboard.summary', { waiting: c.waiting, seen: c.seen }) : t('dashboard.title')}
        actions={
          <>
            <AddToQueue />
            <button onClick={() => setShowNew(true)} className="btn btn-primary">
              <UserPlus className="h-4 w-4" /> {t('patients.new')}
            </button>
          </>
        }
      />

      <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
        <StatCard icon={Users} tone="blue" label={t('dashboard.today')} value={c?.today ?? '—'} />
        <StatCard icon={Hourglass} tone="amber" label={t('dashboard.waiting')} value={c?.waiting ?? '—'}
          hint={c && c.in_consultation > 0 ? t('dashboard.inConsultation', { n: c.in_consultation }) : undefined} />
        <StatCard icon={CheckCircle2} tone="green" label={t('dashboard.seen')} value={c?.seen ?? '—'}
          hint={c && c.today > 0 ? t('dashboard.done', { pct: done }) : undefined} />
        <StatCard icon={BellRing} tone="violet" label={t('dashboard.followupsDue')} value={c?.followups_due ?? '—'} to="/followups" />
      </div>

      <Card
        title={
          <span className="flex items-center gap-2">
            <ListOrdered className="h-5 w-5 text-brand" /> {t('dashboard.queue')}
          </span>
        }
        actions={
          c && c.today > 0 ? (
            <span className="flex items-center gap-2 text-xs text-slate-500">
              <span className="h-1.5 w-28 overflow-hidden rounded-full bg-slate-100">
                <span className="block h-full rounded-full bg-emerald-500" style={{ width: `${done}%` }} />
              </span>
              {t('dashboard.seenOf', { seen: c.seen, today: c.today })}
            </span>
          ) : undefined
        }
      >
        {isError ? (
          <p className="text-red-600">{error.message}</p>
        ) : !data ? (
          <p className="text-slate-500">{t('common.loading')}</p>
        ) : data.items.length === 0 ? (
          <div className="flex flex-col items-center gap-2 py-10 text-center">
            <span className="flex h-14 w-14 items-center justify-center rounded-2xl bg-slate-100 text-slate-400">
              <Coffee className="h-7 w-7" />
            </span>
            <p className="font-medium text-slate-700">{t('queue.empty')}</p>
            <p className="text-sm text-slate-500">{t('dashboard.emptyHint')}</p>
          </div>
        ) : (
          <QueueTable items={data.items} />
        )}
      </Card>

      {showNew && <PatientFormDialog onClose={() => setShowNew(false)} />}
    </div>
  )
}

/** Search box that adds an existing patient to today's queue. */
function AddToQueue() {
  const { t } = useTranslation()
  const [q, setQ] = useState('')
  const [type, setType] = useState<VisitType>('new')
  const query = useDeferredValue(q.trim())
  const { data } = usePatients(query, 1, query.length >= 2)
  const add = useAddToQueue()
  const open = query.length >= 2 && !!data

  return (
    <div className="relative flex w-full gap-2 sm:w-auto">
      <input
        value={q}
        onChange={(e) => {
          setQ(e.target.value)
          add.reset()
        }}
        placeholder={`🔍 ${t('queue.searchToAdd')}`}
        className="w-full min-w-0 rounded-lg border px-3 py-2 text-sm sm:w-64"
      />
      <select
        value={type}
        onChange={(e) => setType(e.target.value as VisitType)}
        className="shrink-0 rounded-lg border bg-white px-2 py-2 text-sm"
        aria-label={t('queue.visitType')}
      >
        {(['new', 'followup', 'report'] as const).map((v) => (
          <option key={v} value={v}>
            {t(`visits.type.${v}`)}
          </option>
        ))}
      </select>
      {open && (
        <div className="absolute top-full z-20 mt-1 w-96 max-w-[90vw] rounded-md border bg-white shadow-lg">
          {add.isError && <p className="px-3 py-2 text-sm text-red-600">{add.error.message}</p>}
          {data.items.length === 0 ? (
            <p className="px-3 py-2 text-sm text-slate-500">{t('patients.empty')}</p>
          ) : (
            <ul className="max-h-72 divide-y overflow-y-auto">
              {data.items.map((p) => (
                <li key={p.id} className="flex items-center gap-2 px-3 py-2 text-sm">
                  <div className="min-w-0 flex-1">
                    <div className="font-medium">{fullName(p)}</div>
                    <div className="text-xs text-slate-500">
                      {p.patient_code} · {ageText(p.dob)} / {p.sex} · {p.phone}
                    </div>
                  </div>
                  <button
                    onClick={() => add.mutate({ patient_id: p.id, visit_type: type }, { onSuccess: () => setQ('') })}
                    disabled={add.isPending}
                    className="rounded-md bg-brand px-2 py-1 text-xs text-white disabled:opacity-60"
                  >
                    ➕ {t('profile.addToQueue')}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  )
}

function QueueTable({ items }: { items: QueueEntry[] }) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const role = useAuth((s) => s.user?.role)
  const update = useUpdateQueue()
  const [vitalsFor, setVitalsFor] = useState<QueueEntry | null>(null)
  const [billFor, setBillFor] = useState<QueueEntry | null>(null)
  const [dragId, setDragId] = useState<number | null>(null)
  const waiting = items.filter((v) => v.status === 'waiting')

  const move = (id: number, position: number) => {
    if (position >= 0 && position < waiting.length) update.mutate({ id, position })
  }
  const drop = (e: DragEvent, target: QueueEntry) => {
    e.preventDefault()
    if (dragId !== null && dragId !== target.id) move(dragId, waiting.indexOf(target))
    setDragId(null)
  }

  const startRx = async (v: QueueEntry) => {
    if (v.status === 'waiting') await update.mutateAsync({ id: v.id, status: 'in_consultation' })
    navigate(`/visits/${v.id}/prescription`)
  }

  return (
    <>
      {update.isError && <p className="mb-2 text-sm text-red-600">{update.error.message}</p>}
      <div className="overflow-x-auto">
        <table className="w-full text-left text-sm">
          <thead className="border-b text-slate-500">
            <tr>
              <th className="py-2 pr-3">#</th>
              <th className="py-2 pr-3">{t('patients.name')}</th>
              <th className="py-2 pr-3">{t('patients.ageSex')}</th>
              <th className="py-2 pr-3">{t('patients.phone')}</th>
              <th className="py-2 pr-3">{t('queue.type')}</th>
              <th className="py-2 pr-3">{t('queue.status')}</th>
              <th className="py-2 pr-3">{t('queue.action')}</th>
            </tr>
          </thead>
          <tbody>
            {items.map((v) => {
              const isWaiting = v.status === 'waiting'
              const idx = waiting.indexOf(v)
              const open = isWaiting || v.status === 'in_consultation'
              return (
                <tr
                  key={v.id}
                  draggable={isWaiting}
                  onDragStart={() => setDragId(v.id)}
                  onDragEnd={() => setDragId(null)}
                  onDragOver={(e) => isWaiting && dragId !== null && e.preventDefault()}
                  onDrop={(e) => drop(e, v)}
                  className={`border-b last:border-0 ${v.status === 'cancelled' ? 'text-slate-400' : ''} ${
                    dragId === v.id ? 'opacity-50' : ''
                  } ${v.status === 'in_consultation' ? 'bg-brand-50/70' : ''}`}
                >
                  <td className="py-3 pr-3 whitespace-nowrap">
                    <span className="flex items-center gap-1">
                      {isWaiting ? (
                        <GripVertical className="h-4 w-4 cursor-grab text-slate-300" aria-label={t('queue.drag')} />
                      ) : (
                        <span className="w-4" />
                      )}
                      <span
                        className={`flex h-8 w-8 items-center justify-center rounded-full text-sm font-bold tabular-nums ${
                          v.status === 'in_consultation'
                            ? 'bg-brand text-white'
                            : v.status === 'waiting'
                              ? 'bg-amber-100 text-amber-800'
                              : 'bg-slate-100 text-slate-500'
                        }`}
                      >
                        {v.queue_serial}
                      </span>
                    </span>
                  </td>
                  <td className="py-3 pr-3">
                    <Link to={`/patients/${v.patient.id}`} className="font-medium text-slate-900 hover:text-brand">
                      {fullName(v.patient)}
                    </Link>
                    {v.has_allergy && (
                      <span
                        className="ml-1.5 inline-flex items-center gap-0.5 rounded-full bg-red-50 px-1.5 text-[11px] font-semibold text-red-700 ring-1 ring-red-200"
                        title={t('profile.allergies')}
                      >
                        <AlertTriangle className="h-3 w-3" /> {t('dashboard.allergy')}
                      </span>
                    )}
                    <span className="block text-xs text-slate-500">{v.patient.patient_code}</span>
                  </td>
                  <td className="py-3 pr-3 whitespace-nowrap">
                    {ageText(v.patient.dob)} / {v.patient.sex}
                  </td>
                  <td className="py-3 pr-3 whitespace-nowrap tabular-nums">{v.patient.phone}</td>
                  <td className="py-2 pr-3">{t(`visits.type.${v.visit_type}`)}</td>
                  <td className="py-2 pr-3">
                    <StatusBadge status={v.status} />
                  </td>
                  <td className="py-2 pr-3">
                    <div className="flex flex-wrap items-center gap-1">
                      {role === 'doctor' && v.status !== 'cancelled' && (
                        <button
                          onClick={() => startRx(v)}
                          disabled={update.isPending}
                          className={`rounded-lg px-2.5 py-1 text-xs font-medium ${
                            open ? 'bg-brand text-white shadow-xs hover:bg-brand-dark' : 'border border-brand/30 text-brand hover:bg-brand-50'
                          } disabled:opacity-60`}
                        >
                          {isWaiting ? t('visits.startRx') : v.status === 'seen' ? t('queue.open') : `℞ ${t('queue.continue')}`}
                        </button>
                      )}
                      {role !== 'admin' && v.status !== 'cancelled' && (
                        <button onClick={() => setVitalsFor(v)} className="rounded-md border px-2 py-1 text-xs">
                          {v.has_vitals ? '✔ ' : '➕ '}
                          {t('vitals.short')}
                        </button>
                      )}
                      {v.status !== 'cancelled' && (
                        <button
                          onClick={() => setBillFor(v)}
                          className={`rounded-md border px-2 py-1 text-xs ${
                            v.bill_status === 'paid'
                              ? 'border-green-600 text-green-700'
                              : v.bill_status === 'due'
                                ? 'border-amber-600 text-amber-700'
                                : ''
                          }`}
                        >
                          {v.bill_status === 'paid'
                            ? `✔ ${t('billing.statusPaid')}`
                            : v.bill_status === 'due'
                              ? `৳ ${t('billing.statusDue')}`
                              : `💳 ${t('billing.short')}`}
                        </button>
                      )}
                      {isWaiting && (
                        <>
                          <button
                            onClick={() => move(v.id, idx - 1)}
                            disabled={idx === 0 || update.isPending}
                            className="rounded-md border px-1.5 py-1 text-xs disabled:opacity-30"
                            aria-label={t('queue.moveUp')}
                          >
                            ▲
                          </button>
                          <button
                            onClick={() => move(v.id, idx + 1)}
                            disabled={idx === waiting.length - 1 || update.isPending}
                            className="rounded-md border px-1.5 py-1 text-xs disabled:opacity-30"
                            aria-label={t('queue.moveDown')}
                          >
                            ▼
                          </button>
                        </>
                      )}
                      {open && (
                        <button
                          onClick={() => update.mutate({ id: v.id, status: 'cancelled' })}
                          className="px-1.5 text-red-600"
                          aria-label={t('queue.cancel')}
                          title={t('queue.cancel')}
                        >
                          ✕
                        </button>
                      )}
                      {v.status === 'cancelled' && (
                        <button
                          onClick={() => update.mutate({ id: v.id, status: 'waiting' })}
                          className="rounded-md border px-2 py-1 text-xs"
                        >
                          ↺ {t('queue.restore')}
                        </button>
                      )}
                    </div>
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
      {billFor && (
        <BillDialog
          visitId={billFor.id}
          visitDate={billFor.visit_date}
          visitType={billFor.visit_type}
          patient={billFor.patient}
          onClose={() => setBillFor(null)}
        />
      )}
      {vitalsFor && (
        <VitalsDialog visitId={vitalsFor.id} patientId={vitalsFor.patient.id} onClose={() => setVitalsFor(null)} />
      )}
    </>
  )
}
