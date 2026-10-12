import { useState } from 'react'
import { Link, useSearchParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { addDays, format, parseISO, startOfWeek } from 'date-fns'
import type { Appointment, AppointmentStatus } from '../../api/types'
import { Card } from '../../components/Card'
import { fmtDate, todayIso } from '../../utils/date'
import { fullName } from '../../utils/patient'
import { useAppointments, useSlots, useUpdateAppointment } from './api'
import { BookDialog } from './BookDialog'

type View = 'day' | 'week' | 'list'
const iso = (d: Date) => format(d, 'yyyy-MM-dd')
const LIST_DAYS = 14

const statusClass: Record<AppointmentStatus, string> = {
  booked: 'bg-blue-100 text-blue-800',
  arrived: 'bg-amber-100 text-amber-800',
  seen: 'bg-green-100 text-green-800',
  no_show: 'bg-red-100 text-red-800',
  cancelled: 'bg-slate-200 text-slate-600',
}

const dotClass: Record<AppointmentStatus, string> = {
  booked: 'bg-blue-600',
  arrived: 'bg-amber-500',
  seen: 'bg-green-600',
  no_show: 'bg-red-600',
  cancelled: 'bg-slate-400',
}

function AppointmentBadge({ status }: { status: AppointmentStatus }) {
  const { t } = useTranslation()
  return (
    <span className={`whitespace-nowrap rounded px-2 text-xs ${statusClass[status]}`}>
      {status === 'seen' ? '✔ ' : ''}
      {t(`appointments.status.${status}`)}
    </span>
  )
}

/** Appointments (§5.6): day and week calendar plus a list, with booking. */
export function AppointmentsPage() {
  const { t } = useTranslation()
  const [params, setParams] = useSearchParams()
  const view = (params.get('view') as View) || 'day'
  const date = params.get('date') || todayIso()
  const bookFor = params.get('book') ?? undefined
  const [booking, setBooking] = useState<{ time?: string } | null>(bookFor ? {} : null)
  const [moving, setMoving] = useState<Appointment | null>(null)

  const go = (patch: Record<string, string | null>) => {
    const next = new URLSearchParams(params)
    for (const [k, v] of Object.entries(patch)) {
      if (v === null) next.delete(k)
      else next.set(k, v)
    }
    setParams(next, { replace: true })
  }

  // Bangladesh week: Saturday to Friday.
  const weekStart = startOfWeek(parseISO(date), { weekStartsOn: 6 })
  const [from, to] =
    view === 'day'
      ? [date, date]
      : view === 'week'
        ? [iso(weekStart), iso(addDays(weekStart, 6))]
        : [date, iso(addDays(parseISO(date), LIST_DAYS - 1))]
  const { data, isError, error } = useAppointments(from, to)
  const step = view === 'day' ? 1 : 7
  const shift = (days: number) => go({ date: iso(addDays(parseISO(date), days)) })

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-2">
        <h1 className="text-lg font-semibold">{t('nav.appointments')}</h1>
        <div className="flex overflow-hidden rounded border bg-white text-sm" role="tablist">
          {(['day', 'week', 'list'] as const).map((v) => (
            <button
              key={v}
              role="tab"
              aria-selected={view === v}
              onClick={() => go({ view: v })}
              className={`px-3 py-1.5 ${view === v ? 'bg-brand text-white' : 'text-slate-600'}`}
            >
              {t(`appointments.view.${v}`)}
            </button>
          ))}
        </div>
        <div className="flex items-center gap-1 text-sm">
          <button onClick={() => shift(-step)} className="rounded border bg-white px-2 py-1" aria-label={t('common.prev')}>
            ◀
          </button>
          <button onClick={() => go({ date: todayIso() })} className="rounded border bg-white px-2 py-1">
            {t('billing.today')}
          </button>
          <button onClick={() => shift(step)} className="rounded border bg-white px-2 py-1" aria-label={t('common.next')}>
            ▶
          </button>
          <input
            type="date"
            value={date}
            onChange={(e) => e.target.value && go({ date: e.target.value })}
            className="rounded border px-2 py-1"
            aria-label={t('appointments.date')}
          />
        </div>
        <button onClick={() => setBooking({})} className="ml-auto rounded bg-brand px-3 py-1.5 text-sm text-white">
          📅 {t('appointments.book')}
        </button>
      </div>

      {isError && <p className="text-red-600">{error.message}</p>}
      {!data ? (
        <p className="text-slate-500">{t('common.loading')}</p>
      ) : view === 'day' ? (
        <DayView date={date} items={data} onBook={(time) => setBooking({ time })} onMove={setMoving} />
      ) : view === 'week' ? (
        <WeekView start={weekStart} items={data} onOpenDay={(d) => go({ view: 'day', date: d })} />
      ) : (
        <ListView items={data} onMove={setMoving} />
      )}

      {booking && (
        <BookDialog
          mode="book"
          patientId={bookFor}
          date={date < todayIso() ? todayIso() : date}
          time={booking.time}
          onClose={() => {
            setBooking(null)
            if (bookFor) go({ book: null })
          }}
        />
      )}
      {moving && <BookDialog mode="move" appointment={moving} onClose={() => setMoving(null)} />}
    </div>
  )
}

function DayView({
  date,
  items,
  onBook,
  onMove,
}: {
  date: string
  items: Appointment[]
  onBook: (time: string) => void
  onMove: (a: Appointment) => void
}) {
  const { t } = useTranslation()
  const { data: day } = useSlots(date)
  const slotTimes = day?.slots ?? []
  const times = [...new Set([...slotTimes.map((s) => s.time), ...items.map((a) => a.time)])].sort()
  const active = items.filter((a) => a.status !== 'cancelled' && a.status !== 'no_show').length

  return (
    <Card
      title={fmtDate(date)}
      actions={
        <span className="text-sm text-slate-500">
          {day?.daily_limit
            ? t('appointments.bookedOf', { n: active, limit: day.daily_limit })
            : t('appointments.nBooked', { count: active })}
        </span>
      }
    >
      {times.length === 0 ? (
        <p className="text-slate-500">{day?.free_time ? t('appointments.noneDay') : t('appointments.closed')}</p>
      ) : (
        <ul className="divide-y">
          {times.map((time) => {
            const here = items.filter((a) => a.time === time)
            const slot = slotTimes.find((s) => s.time === time)
            const free = slot && !slot.taken && !slot.past
            return (
              <li key={time} className="flex flex-wrap items-start gap-3 py-2">
                <span className={`w-12 pt-1 text-sm font-medium ${slot?.past ? 'text-slate-400' : ''}`}>{time}</span>
                <div className="min-w-0 flex-1 space-y-1">
                  {here.map((a) => (
                    <AppointmentRow key={a.id} a={a} onMove={onMove} />
                  ))}
                  {free && (
                    <button onClick={() => onBook(time)} className="rounded border border-dashed px-2 py-1 text-xs text-slate-500 hover:border-brand hover:text-brand">
                      ➕ {t('appointments.book')}
                    </button>
                  )}
                </div>
              </li>
            )
          })}
        </ul>
      )}
    </Card>
  )
}

function AppointmentRow({ a, onMove, showDate }: { a: Appointment; onMove: (a: Appointment) => void; showDate?: boolean }) {
  const { t } = useTranslation()
  const dim = a.status === 'cancelled' || a.status === 'no_show'
  return (
    <div className={`flex flex-wrap items-center gap-2 text-sm ${dim ? 'text-slate-400' : ''}`}>
      {showDate && (
        <span className="w-28 whitespace-nowrap">
          {fmtDate(a.date)} <b>{a.time}</b>
        </span>
      )}
      <Link to={`/patients/${a.patient.id}`} className={dim ? 'line-through' : 'font-medium text-brand hover:underline'}>
        {fullName(a.patient)}
      </Link>
      <span className="text-xs text-slate-500">
        {a.patient.patient_code} · {a.patient.phone}
      </span>
      <span className="text-xs">{t(`visits.type.${a.visit_type}`)}</span>
      <AppointmentBadge status={a.status} />
      {a.sms_sent_at && (
        <span className="text-xs text-slate-500" title={t('appointments.smsSent')}>
          💬✔
        </span>
      )}
      {a.notes && <span className="text-xs text-slate-500">“{a.notes}”</span>}
      <AppointmentActions a={a} onMove={onMove} />
    </div>
  )
}

/** Arrived (→ queue), move, no-show, cancel, restore. */
function AppointmentActions({ a, onMove }: { a: Appointment; onMove: (a: Appointment) => void }) {
  const { t } = useTranslation()
  const update = useUpdateAppointment()
  const today = todayIso()
  const set = (status: AppointmentStatus) => update.mutate({ id: a.id, status })
  const btn = 'rounded border px-2 py-0.5 text-xs disabled:opacity-50'

  return (
    <span className="ml-auto flex flex-wrap items-center gap-1">
      {a.status === 'booked' && (
        <>
          {a.date === today && (
            <button onClick={() => set('arrived')} disabled={update.isPending} className={`${btn} border-brand bg-brand text-white`}>
              ✔ {t('appointments.arrived')}
            </button>
          )}
          <button onClick={() => onMove(a)} className={btn}>
            ↔ {t('appointments.move')}
          </button>
          {a.date <= today && (
            <button onClick={() => set('no_show')} disabled={update.isPending} className={btn}>
              {t('appointments.status.no_show')}
            </button>
          )}
          <button
            onClick={() => set('cancelled')}
            disabled={update.isPending}
            className="px-1.5 text-red-600"
            aria-label={t('queue.cancel')}
            title={t('queue.cancel')}
          >
            ✕
          </button>
        </>
      )}
      {(a.status === 'cancelled' || a.status === 'no_show') && a.date >= today && (
        <button onClick={() => set('booked')} disabled={update.isPending} className={btn}>
          ↺ {t('queue.restore')}
        </button>
      )}
      {(a.status === 'arrived' || a.status === 'seen') && a.date === today && (
        <Link to="/dashboard" className="text-xs text-brand hover:underline">
          {t('appointments.inQueue')} →
        </Link>
      )}
      {update.isError && <span className="w-full text-xs text-red-600">{update.error.message}</span>}
    </span>
  )
}

function WeekView({ start, items, onOpenDay }: { start: Date; items: Appointment[]; onOpenDay: (d: string) => void }) {
  const { t } = useTranslation()
  const days = Array.from({ length: 7 }, (_, i) => addDays(start, i))
  const today = todayIso()
  return (
    <div className="grid gap-2 md:grid-cols-7">
      {days.map((d) => {
        const key = iso(d)
        const mine = items.filter((a) => a.date === key)
        const active = mine.filter((a) => a.status !== 'cancelled' && a.status !== 'no_show')
        return (
          <section key={key} className={`rounded-lg bg-white shadow-sm ${key === today ? 'ring-2 ring-brand' : ''}`}>
            <button
              onClick={() => onOpenDay(key)}
              className="flex w-full items-baseline gap-2 border-b px-3 py-2 text-left hover:bg-slate-50"
            >
              <span className="font-semibold">{t(`days.${d.getDay()}`)}</span>
              <span className="text-sm text-slate-500">{format(d, 'd MMM')}</span>
              <span className="ml-auto text-xs text-slate-500">{active.length || ''}</span>
            </button>
            <ul className="space-y-1 p-2 text-xs">
              {mine.length === 0 && <li className="text-slate-400">—</li>}
              {mine.map((a) => (
                <li key={a.id} className={`flex items-center gap-1 ${a.status === 'cancelled' || a.status === 'no_show' ? 'text-slate-400 line-through' : ''}`}>
                  <b className="tabular-nums">{a.time}</b>
                  <span className="truncate">{fullName(a.patient)}</span>
                  <span className={`ml-auto h-2 w-2 shrink-0 rounded-full ${dotClass[a.status]}`} title={t(`appointments.status.${a.status}`)} />
                </li>
              ))}
            </ul>
          </section>
        )
      })}
    </div>
  )
}

function ListView({ items, onMove }: { items: Appointment[]; onMove: (a: Appointment) => void }) {
  const { t } = useTranslation()
  return (
    <Card title={t('appointments.nextDays', { count: LIST_DAYS })}>
      {items.length === 0 ? (
        <p className="text-slate-500">{t('appointments.noneRange')}</p>
      ) : (
        <div className="divide-y">
          {items.map((a) => (
            <div key={a.id} className="py-2">
              <AppointmentRow a={a} onMove={onMove} showDate />
            </div>
          ))}
        </div>
      )}
    </Card>
  )
}
