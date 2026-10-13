import { useDeferredValue, useState, type FormEvent } from 'react'
import { useTranslation } from 'react-i18next'
import type { Appointment, PatientBrief, VisitType } from '../../api/types'
import { ageText } from '../../utils/age'
import { todayIso } from '../../utils/date'
import { fullName } from '../../utils/patient'
import { usePatient, usePatients } from '../patients/api'
import { PatientFormDialog } from '../patients/PatientFormDialog'
import { useBookAppointment, useSlots, useUpdateAppointment } from './api'

type Props =
  | { mode: 'book'; patientId?: string; date: string; time?: string; onClose: () => void }
  | { mode: 'move'; appointment: Appointment; onClose: () => void }

/** Book an appointment (§5.6), or move one to another slot (reschedule). */
export function BookDialog(props: Props) {
  const { t } = useTranslation()
  const moving = props.mode === 'move' ? props.appointment : null
  const preset = usePatient(props.mode === 'book' ? (props.patientId ?? '') : '')
  const [picked, setPicked] = useState<PatientBrief | null>(null)
  const patient = moving?.patient ?? picked ?? preset.data ?? null
  const [date, setDate] = useState(moving?.date ?? (props.mode === 'book' ? props.date : todayIso()))
  const [time, setTime] = useState(props.mode === 'book' ? (props.time ?? '') : '')
  const [type, setType] = useState<VisitType>(moving?.visit_type ?? 'followup')
  const [notes, setNotes] = useState('')
  const slots = useSlots(date)
  const book = useBookAppointment()
  const move = useUpdateAppointment()
  const mutation = moving ? move : book

  const submit = async (e: FormEvent) => {
    e.preventDefault()
    if (!patient || !time) return
    if (moving) await move.mutateAsync({ id: moving.id, date, time })
    else await book.mutateAsync({ patient_id: patient.id, date, time, visit_type: type, notes })
    props.onClose()
  }

  const day = slots.data
  const open = day && (day.free_time || day.slots.length > 0)
  const full = !!day?.daily_limit && day.booked >= day.daily_limit

  return (
    <div className="overlay">
      <form onSubmit={submit} className="w-full max-w-lg space-y-4 dialog">
        <h2 className="text-lg font-semibold">
          {moving ? `↔ ${t('appointments.move')}` : `📅 ${t('appointments.book')}`}
        </h2>

        {patient ? (
          <div className="flex items-center gap-2 rounded-md bg-infobar px-3 py-2 text-sm">
            <div className="min-w-0 flex-1">
              <b>{fullName(patient)}</b> · {patient.patient_code}
              <div className="text-xs text-slate-600">
                {ageText(patient.dob)} / {patient.sex} · {patient.phone}
              </div>
            </div>
            {!moving && (
              <button type="button" onClick={() => setPicked(null)} className="text-xs text-brand">
                {t('appointments.change')}
              </button>
            )}
          </div>
        ) : (
          <PatientPicker onPick={setPicked} />
        )}

        <div className="grid grid-cols-2 gap-3">
          <label className="text-sm">
            {t('appointments.date')}
            <input
              type="date"
              value={date}
              min={todayIso()}
              onChange={(e) => {
                setDate(e.target.value)
                setTime('')
              }}
              required
              className="mt-1 w-full rounded-md border px-3 py-2"
            />
          </label>
          {!moving && (
            <label className="text-sm">
              {t('queue.visitType')}
              <select
                value={type}
                onChange={(e) => setType(e.target.value as VisitType)}
                className="mt-1 w-full rounded-md border bg-white px-2 py-2"
              >
                {(['new', 'followup', 'report'] as const).map((v) => (
                  <option key={v} value={v}>
                    {t(`visits.type.${v}`)}
                  </option>
                ))}
              </select>
            </label>
          )}
        </div>

        <div className="text-sm">
          <div className="flex items-center">
            {t('appointments.time')}
            {day?.daily_limit && (
              <span className={`ml-auto text-xs ${full ? 'text-red-600' : 'text-slate-500'}`}>
                {t('appointments.bookedOf', { n: day.booked, limit: day.daily_limit })}
              </span>
            )}
          </div>
          {!day ? (
            <p className="mt-1 text-slate-500">{t('common.loading')}</p>
          ) : day.free_time ? (
            <input
              type="time"
              value={time}
              onChange={(e) => setTime(e.target.value)}
              required
              className="mt-1 w-full rounded-md border px-3 py-2"
            />
          ) : !open ? (
            <p className="mt-1 text-slate-500">{t('appointments.closed')}</p>
          ) : (
            <div className="mt-1 grid grid-cols-4 gap-1.5 sm:grid-cols-6" role="radiogroup">
              {day.slots.map((s) => {
                const disabled = s.taken || s.past
                return (
                  <button
                    key={s.time}
                    type="button"
                    role="radio"
                    aria-checked={time === s.time}
                    disabled={disabled}
                    onClick={() => setTime(s.time)}
                    title={s.taken ? t('appointments.taken') : undefined}
                    className={`rounded-md border px-1 py-1.5 text-xs ${
                      time === s.time
                        ? 'border-brand bg-brand text-white'
                        : disabled
                          ? 'bg-slate-100 text-slate-400 line-through'
                          : 'hover:border-brand'
                    }`}
                  >
                    {s.time}
                  </button>
                )
              })}
            </div>
          )}
        </div>

        {!moving && (
          <label className="block text-sm">
            {t('appointments.notes')}
            <input
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              maxLength={255}
              className="mt-1 w-full rounded-md border px-3 py-2"
            />
          </label>
        )}

        {mutation.isError && <p className="text-sm text-red-600">{mutation.error.message}</p>}
        <div className="flex justify-end gap-2">
          <button type="button" onClick={props.onClose} className="btn btn-secondary">
            {t('patients.cancel')}
          </button>
          <button
            disabled={!patient || !time || mutation.isPending}
            className="btn btn-primary"
          >
            {moving ? t('appointments.move') : t('appointments.book')}
          </button>
        </div>
      </form>
    </div>
  )
}

/** Patient search inside the booking dialog, with a link to register a new one. */
function PatientPicker({ onPick }: { onPick: (p: PatientBrief) => void }) {
  const { t } = useTranslation()
  const [q, setQ] = useState('')
  const [registering, setRegistering] = useState(false)
  const query = useDeferredValue(q.trim())
  const { data } = usePatients(query, 1, query.length >= 2)

  return (
    <div className="text-sm">
      <div className="flex gap-2">
        <input
          value={q}
          onChange={(e) => setQ(e.target.value)}
          placeholder={`🔍 ${t('patients.search')}`}
          autoFocus
          className="w-full rounded-md border px-3 py-2"
        />
        <button
          type="button"
          onClick={() => setRegistering(true)}
          className="whitespace-nowrap rounded-md border px-3 py-2 text-brand"
        >
          ➕ {t('patients.new')}
        </button>
      </div>
      {query.length >= 2 && data && (
        <ul className="mt-1 max-h-56 divide-y overflow-y-auto rounded-md border">
          {data.items.length === 0 && <li className="px-3 py-2 text-slate-500">{t('patients.empty')}</li>}
          {data.items.map((p) => (
            <li key={p.id}>
              <button type="button" onClick={() => onPick(p)} className="w-full px-3 py-2 text-left hover:bg-slate-50">
                <b>{fullName(p)}</b>
                <span className="ml-2 text-xs text-slate-500">
                  {p.patient_code} · {ageText(p.dob)} / {p.sex} · {p.phone}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
      {registering && <PatientFormDialog onClose={() => setRegistering(false)} onCreated={onPick} />}
    </div>
  )
}
