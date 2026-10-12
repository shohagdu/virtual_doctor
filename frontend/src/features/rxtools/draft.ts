import { useEffect, useState } from 'react'
import type { PrescriptionBody } from '../../api/types'

interface Draft {
  rxId: number | null
  revision: number
  body: PrescriptionBody
  at: string
}

const key = (visitId: number) => `vd-rx-draft-${visitId}`

function read(visitId: number): Draft | null {
  try {
    const raw = localStorage.getItem(key(visitId))
    return raw ? (JSON.parse(raw) as Draft) : null
  } catch {
    return null
  }
}

export function clearDraft(visitId: number) {
  try {
    localStorage.removeItem(key(visitId))
  } catch {
    /* storage unavailable */
  }
}

/**
 * Keeps unsaved ℞ edits in this browser (§13 "offline save"), so a closed tab, a
 * reload or a dropped connection does not lose them. A draft is offered back only if
 * it was made on the version now on the server; otherwise it is stale and dropped.
 */
export function useLocalDraft(visitId: number, rxId: number | null, revision: number, body: PrescriptionBody, dirty: boolean) {
  const [offer, setOffer] = useState<Draft | null>(() => {
    const d = read(visitId)
    if (!d) return null
    if (d.rxId !== rxId || d.revision !== revision) {
      clearDraft(visitId)
      return null
    }
    return d
  })

  useEffect(() => {
    if (!dirty) return
    const timer = setTimeout(() => {
      try {
        localStorage.setItem(key(visitId), JSON.stringify({ rxId, revision, body, at: new Date().toISOString() }))
      } catch {
        /* storage full or blocked: nothing to do */
      }
    }, 800)
    return () => clearTimeout(timer)
  }, [visitId, rxId, revision, body, dirty])

  return {
    offer,
    dismiss: () => {
      clearDraft(visitId)
      setOffer(null)
    },
    taken: () => setOffer(null),
  }
}
