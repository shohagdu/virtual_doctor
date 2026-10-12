import { useActionState } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { api } from '../../api/client'
import { useMe } from '../../api/me'
import type { Me } from '../../api/types'
import { Card } from '../../components/Card'

const input = 'mt-1 w-full rounded border px-3 py-2'

/** Doctor and clinic details printed on the prescription pad (§12). */
export function SettingsPage() {
  const { t } = useTranslation()
  const { data: me } = useMe()
  if (!me) return <p className="text-slate-500">{t('common.loading')}</p>
  return (
    <div className="grid gap-4 md:grid-cols-2">
      <ProfileForm me={me} />
      {me.user.role !== 'assistant' && <ClinicForm me={me} />}
    </div>
  )
}

type Status = { ok: boolean; message: string } | null

function useSubmit(path: string) {
  const qc = useQueryClient()
  const { t } = useTranslation()
  return useActionState(async (_prev: Status, form: FormData): Promise<Status> => {
    try {
      const me = await api<Me>(path, { method: 'PATCH', body: JSON.stringify(Object.fromEntries(form)) })
      qc.setQueryData(['me'], me)
      return { ok: true, message: t('settings.saved') }
    } catch (e) {
      return { ok: false, message: (e as Error).message }
    }
  }, null)
}

function StatusLine({ status }: { status: Status }) {
  if (!status) return null
  return <p className={`text-sm ${status.ok ? 'text-green-700' : 'text-red-600'}`}>{status.message}</p>
}

function ProfileForm({ me }: { me: Me }) {
  const { t } = useTranslation()
  const [status, submit, pending] = useSubmit('/me')
  const u = me.user
  return (
    <Card title={t('settings.profile')}>
      <form action={submit} className="space-y-3">
        <label className="block text-sm">
          {t('settings.name')} *
          <input name="name" defaultValue={u.name} required className={input} />
        </label>
        <label className="block text-sm">
          {t('settings.phone')}
          <input name="phone" defaultValue={u.phone ?? ''} className={input} />
        </label>
        {u.role === 'doctor' && (
          <>
            <label className="block text-sm">
              {t('settings.degrees')}
              <input name="degrees" defaultValue={u.degrees ?? ''} placeholder="MBBS, FCPS (Medicine)" className={input} />
            </label>
            <label className="block text-sm">
              {t('settings.bmdc')}
              <input name="bmdc_reg_no" defaultValue={u.bmdc_reg_no ?? ''} className={input} />
            </label>
          </>
        )}
        <StatusLine status={status} />
        <button disabled={pending} className="rounded bg-brand px-4 py-2 text-white disabled:opacity-60">
          {t('patients.save')}
        </button>
      </form>
    </Card>
  )
}

function ClinicForm({ me }: { me: Me }) {
  const { t } = useTranslation()
  const [status, submit, pending] = useSubmit('/clinic')
  const c = me.clinic
  return (
    <Card title={t('settings.clinic')}>
      <form action={submit} className="space-y-3">
        <label className="block text-sm">
          {t('settings.clinicName')} *
          <input name="name" defaultValue={c.name} required className={input} />
        </label>
        <label className="block text-sm">
          {t('settings.address')}
          <input name="address" defaultValue={c.address ?? ''} className={input} />
        </label>
        <label className="block text-sm">
          {t('settings.phone')}
          <input name="phone" defaultValue={c.phone ?? ''} className={input} />
        </label>
        <StatusLine status={status} />
        <button disabled={pending} className="rounded bg-brand px-4 py-2 text-white disabled:opacity-60">
          {t('patients.save')}
        </button>
      </form>
    </Card>
  )
}
