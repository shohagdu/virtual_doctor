import { useState, type ReactNode } from 'react'
import { Link } from 'react-router'
import { useTranslation } from 'react-i18next'
import type { Followup, SmsDraft } from '../../api/types'
import { fullName } from '../../utils/patient'
import { smsSegments } from '../../utils/sms'
import { useSendSms, useSmsDraft } from './api'
import { useAiAvailable, useSmsAiDraft } from '../ai/api'

/** Reminder SMS from the clinic's template, reviewed (and editable) before sending (§7.3). */
export function SmsDialog({ followup, onClose }: { followup: Followup; onClose: () => void }) {
  const { data, isError, error } = useSmsDraft(followup.id)
  if (isError) {
    return (
      <Shell onClose={onClose}>
        <p className="text-sm text-red-600">{error.message}</p>
      </Shell>
    )
  }
  if (!data) return null
  return <Form followup={followup} draft={data} onClose={onClose} />
}

function Shell({ onClose, children }: { onClose: () => void; children: ReactNode }) {
  const { t } = useTranslation()
  return (
    <div className="overlay">
      <div className="w-full max-w-md space-y-4 dialog">
        {children}
        <div className="flex justify-end">
          <button onClick={onClose} className="btn btn-secondary">
            {t('common.close')}
          </button>
        </div>
      </div>
    </div>
  )
}

function Form({ followup, draft, onClose }: { followup: Followup; draft: SmsDraft; onClose: () => void }) {
  const { t } = useTranslation()
  const [body, setBody] = useState(draft.body)
  const send = useSendSms()
  const aiAvailable = useAiAvailable()
  const aiDraft = useSmsAiDraft()
  const blocked = !draft.enabled || !draft.phone_ok

  const submit = async () => {
    await send.mutateAsync({ id: followup.id, body })
    onClose()
  }

  return (
    <div className="overlay">
      <div className="w-full max-w-md space-y-4 dialog">
        <div>
          <h2 className="text-lg font-semibold">💬 {t('followups.smsTitle')}</h2>
          <p className="text-sm text-slate-500">
            {fullName(followup.patient)} · {followup.patient.phone}
          </p>
        </div>
        {!draft.enabled && (
          <p className="rounded-md bg-amber-50 px-3 py-2 text-sm text-amber-800">
            {t('followups.smsOff')}{' '}
            <Link to="/settings" className="underline">
              {t('nav.settings')}
            </Link>
          </p>
        )}
        {!draft.phone_ok && (
          <p className="rounded-md bg-amber-50 px-3 py-2 text-sm text-amber-800">{t('followups.notMobile')}</p>
        )}
        <label className="block text-sm">
          <textarea
            value={body}
            onChange={(e) => setBody(e.target.value)}
            rows={5}
            maxLength={480}
            className="w-full rounded-md border px-3 py-2"
          />
          <span className="text-xs text-slate-500">
            {t('settings.smsCount', { chars: [...body].length, parts: smsSegments(body) })}
          </span>
        </label>
        {aiAvailable && (
          <button
            type="button"
            onClick={async () => setBody((await aiDraft.mutateAsync(followup.id)).body)}
            disabled={aiDraft.isPending}
            className="rounded-md border border-indigo-300 px-2 py-1 text-xs text-indigo-700 disabled:opacity-50"
          >
            ✨ {aiDraft.isPending ? t('ai.writing') : t('ai.rewordSms')}
          </button>
        )}
        {aiDraft.isError && <p className="text-sm text-red-600">{aiDraft.error.message}</p>}
        {send.isError && <p className="text-sm text-red-600">{send.error.message}</p>}
        <div className="flex justify-end gap-2">
          <button onClick={onClose} className="btn btn-secondary">
            {t('patients.cancel')}
          </button>
          <button
            onClick={submit}
            disabled={blocked || !body.trim() || send.isPending}
            className="btn btn-primary"
          >
            {t('followups.send')}
          </button>
        </div>
      </div>
    </div>
  )
}
