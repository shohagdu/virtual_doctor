import { useState, type ComponentType } from 'react'
import { useTranslation } from 'react-i18next'
import { Baby, HeartPulse, History, House, Pill, ShieldAlert, Syringe, Users } from 'lucide-react'
import type { MedicalProfile, Sex } from '../../api/types'
import { ListSection } from '../../components/ListSection'
import { useClinicSettings } from '../settings/api'
import { useSaveMedicalProfile } from './api'

type TextKey = 'past_history' | 'family_history' | 'social_history' | 'gynae_obs_history'

/** Left panel of the patient profile: persistent data carried into every prescription. */
export function MedicalProfilePanel({
  patientId,
  profile,
  sex,
}: {
  patientId: string
  profile: MedicalProfile
  sex: Sex
}) {
  const { t } = useTranslation()
  const save = useSaveMedicalProfile(patientId)
  // Offered items come from Settings → Prescription lists.
  const lists = useClinicSettings().data?.rx_lists
  // Saved immediately on every ＋/✕; text areas save when they lose focus.
  const update = (patch: Partial<MedicalProfile>) => save.mutate({ ...profile, ...patch })

  const textAreas: [TextKey, string, ComponentType<{ className?: string }>][] = [
    ['past_history', t('profile.pastHistory'), History],
    ['family_history', t('profile.familyHistory'), Users],
    ['social_history', t('profile.socialHistory'), House],
  ]
  if (sex === 'F') textAreas.push(['gynae_obs_history', t('profile.gynaeObs'), Baby])

  return (
    <aside className="space-y-5 rounded-2xl border border-slate-200/80 bg-panel p-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xs font-semibold tracking-wide text-slate-500 uppercase">{t('profile.medicalProfile')}</h2>
        {save.isPending && <span className="text-xs text-slate-400">{t('profile.saving')}</span>}
      </div>
      <ListSection
        title={t('profile.allergies')}
        items={profile.allergies}
        onChange={(allergies) => update({ allergies })}
        placeholder="Penicillin"
        icon={ShieldAlert}
        tone="danger"
      />
      <ListSection
        title={t('profile.chronic')}
        items={profile.chronic_conditions}
        onChange={(chronic_conditions) => update({ chronic_conditions })}
        suggestions={lists?.chronic_conditions}
        icon={HeartPulse}
        tone="chip"
      />
      <ListSection
        title={t('profile.medicines')}
        items={profile.long_term_medicines}
        onChange={(long_term_medicines) => update({ long_term_medicines })}
        icon={Pill}
      />
      {textAreas.map(([key, label, icon]) => (
        <HistoryText key={key} label={label} icon={icon} value={profile[key] ?? ''} onSave={(v) => update({ [key]: v || null })} />
      ))}
      <ListSection
        title={t('profile.vaccination')}
        items={profile.vaccination}
        onChange={(vaccination) => update({ vaccination })}
        icon={Syringe}
        tone="chip"
      />
      {save.isError && <p className="text-xs text-red-600">{save.error.message}</p>}
    </aside>
  )
}

function HistoryText({
  label,
  icon: Icon,
  value,
  onSave,
}: {
  label: string
  icon: ComponentType<{ className?: string }>
  value: string
  onSave: (v: string) => void
}) {
  const { t } = useTranslation()
  const [text, setText] = useState(value)
  return (
    <label className="block text-sm font-semibold text-slate-800">
      <span className="flex items-center gap-2">
        <span className="flex h-6 w-6 items-center justify-center rounded-md bg-white text-slate-500 ring-1 ring-slate-200">
          <Icon className="h-3.5 w-3.5" />
        </span>
        {label}
      </span>
      <textarea
        value={text}
        onChange={(e) => setText(e.target.value)}
        onBlur={() => text.trim() !== value && onSave(text.trim())}
        rows={2}
        placeholder={t('profile.notesPlaceholder')}
        className="mt-1.5 w-full resize-y rounded-lg border bg-white px-2.5 py-1.5 text-sm font-normal"
      />
    </label>
  )
}
