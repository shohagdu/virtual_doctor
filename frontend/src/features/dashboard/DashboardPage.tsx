import { useDeferredValue, useState, type DragEvent } from 'react'
import { Link, useNavigate } from 'react-router'
import { useTranslation } from 'react-i18next'
import type { QueueEntry, VisitType } from '../../api/types'
import { Card } from '../../components/Card'
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
  const { data, isError, error } = useQueueToday()
  const [showNew, setShowNew] = useState(false)
  const c = data?.counters

  const counters = [
    { key: 'dashboard.today', value: c?.today },
    { key: 'dashboard.waiting', value: c?.waiting },
    { key: 'dashboard.seen', value: c?.seen },
    { key: 'dashboard.followupsDue', value: c?.followups_due, to: '/followups' },
  ]

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-2">
        <h1 className="text-lg font-semibold">{t('dashboard.title')}</h1>
        <div className="ml-auto flex flex-wrap gap-2">
          <AddToQueue />
          <button onClick={() => setShowNew(true)} className="rounded bg-brand px-3 py-1.5 text-sm text-white">
            ➕ {t('patients.new')}
          </button>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        {counters.map((x) => {
          const body = (
            <>
              <div className="text-sm text-slate-500">{t(x.key)}</div>
              <div className="text-2xl font-semibold">{x.value ?? '—'}</div>
            </>
          )
          return x.to ? (
            <Link key={x.key} to={x.to} className="rounded-lg bg-white p-4 shadow-sm hover:bg-slate-50">
              {body}
            </Link>
          ) : (
            <div key={x.key} className="rounded-lg bg-white p-4 shadow-sm">
              {body}
            </div>
          )
        })}
      </div>

      <Card title={t('dashboard.queue')}>
        {isError ? (
          <p className="text-red-600">{error.message}</p>
        ) : !data ? (
          <p className="text-slate-500">{t('common.loading')}</p>
        ) : data.items.length === 0 ? (
          <p className="text-slate-500">{t('queue.empty')}</p>
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
    <div className="relative flex gap-2">
      <input
        value={q}
        onChange={(e) => {
          setQ(e.target.value)
          add.reset()
        }}
        placeholder={`🔍 ${t('queue.searchToAdd')}`}
        className="w-64 rounded border px-3 py-1.5 text-sm"
      />
      <select
        value={type}
        onChange={(e) => setType(e.target.value as VisitType)}
        className="rounded border bg-white px-2 py-1.5 text-sm"
        aria-label={t('queue.visitType')}
      >
        {(['new', 'followup', 'report'] as const).map((v) => (
          <option key={v} value={v}>
            {t(`visits.type.${v}`)}
          </option>
        ))}
      </select>
      {open && (
        <div className="absolute top-full z-20 mt-1 w-96 max-w-[90vw] rounded border bg-white shadow-lg">
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
                    className="rounded bg-brand px-2 py-1 text-xs text-white disabled:opacity-60"
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
                  } ${v.status === 'in_consultation' ? 'bg-blue-50' : ''}`}
                >
                  <td className="py-2 pr-3 whitespace-nowrap">
                    {isWaiting && (
                      <span className="mr-1 cursor-grab text-slate-400" title={t('queue.drag')}>
                        ⠿
                      </span>
                    )}
                    {v.queue_serial}
                  </td>
                  <td className="py-2 pr-3">
                    <Link to={`/patients/${v.patient.id}`} className="text-brand hover:underline">
                      {fullName(v.patient)}
                    </Link>
                    {v.has_allergy && (
                      <span className="ml-1 rounded bg-red-600 px-1 text-xs text-white" title={t('profile.allergies')}>
                        ⚠
                      </span>
                    )}
                  </td>
                  <td className="py-2 pr-3 whitespace-nowrap">
                    {ageText(v.patient.dob)} / {v.patient.sex}
                  </td>
                  <td className="py-2 pr-3">{v.patient.phone}</td>
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
                          className={`rounded px-2 py-1 text-xs ${
                            open ? 'bg-brand text-white' : 'border text-brand'
                          } disabled:opacity-60`}
                        >
                          {isWaiting ? t('visits.startRx') : v.status === 'seen' ? t('queue.open') : `℞ ${t('queue.continue')}`}
                        </button>
                      )}
                      {role !== 'admin' && v.status !== 'cancelled' && (
                        <button onClick={() => setVitalsFor(v)} className="rounded border px-2 py-1 text-xs">
                          {v.has_vitals ? '✔ ' : '➕ '}
                          {t('vitals.short')}
                        </button>
                      )}
                      {v.status !== 'cancelled' && (
                        <button
                          onClick={() => setBillFor(v)}
                          className={`rounded border px-2 py-1 text-xs ${
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
                            className="rounded border px-1.5 py-1 text-xs disabled:opacity-30"
                            aria-label={t('queue.moveUp')}
                          >
                            ▲
                          </button>
                          <button
                            onClick={() => move(v.id, idx + 1)}
                            disabled={idx === waiting.length - 1 || update.isPending}
                            className="rounded border px-1.5 py-1 text-xs disabled:opacity-30"
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
                          className="rounded border px-2 py-1 text-xs"
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
