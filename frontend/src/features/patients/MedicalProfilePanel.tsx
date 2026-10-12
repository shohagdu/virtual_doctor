import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { MedicalProfile, Sex } from '../../api/types'
import { ListSection } from '../../components/ListSection'
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
  // Saved immediately on every ➕/✕; text areas save when they lose focus.
  const update = (patch: Partial<MedicalProfile>) => save.mutate({ ...profile, ...patch })

  const textAreas: [TextKey, string][] = [
    ['past_history', t('profile.pastHistory')],
    ['family_history', t('profile.familyHistory')],
    ['social_history', t('profile.socialHistory')],
  ]
  if (sex === 'F') textAreas.push(['gynae_obs_history', t('profile.gynaeObs')])

  return (
    <aside className="space-y-4 rounded-lg bg-panel p-4">
      <ListSection
        title={t('profile.allergies')}
        items={profile.allergies}
        onChange={(allergies) => update({ allergies })}
        placeholder="Penicillin"
      />
      <ListSection
        title={t('profile.chronic')}
        items={profile.chronic_conditions}
        onChange={(chronic_conditions) => update({ chronic_conditions })}
        suggestions={['Diabetes Mellitus', 'Hypertension', 'Asthma', 'CKD', 'IHD', 'Hypothyroidism', 'COPD']}
      />
      <ListSection
        title={t('profile.medicines')}
        items={profile.long_term_medicines}
        onChange={(long_term_medicines) => update({ long_term_medicines })}
      />
      {textAreas.map(([key, label]) => (
        <HistoryText key={key} label={label} value={profile[key] ?? ''} onSave={(v) => update({ [key]: v || null })} />
      ))}
      <ListSection
        title={t('profile.vaccination')}
        items={profile.vaccination}
        onChange={(vaccination) => update({ vaccination })}
      />
      {save.isError && <p className="text-xs text-red-600">{save.error.message}</p>}
    </aside>
  )
}

function HistoryText({ label, value, onSave }: { label: string; value: string; onSave: (v: string) => void }) {
  const [text, setText] = useState(value)
  return (
    <label className="block text-sm font-semibold">
      {label}
      <textarea
        value={text}
        onChange={(e) => setText(e.target.value)}
        onBlur={() => text.trim() !== value && onSave(text.trim())}
        rows={2}
        className="mt-1 w-full rounded border bg-white px-2 py-1 font-normal"
      />
    </label>
  )
}
