import { useAuth } from '../features/auth/authStore'
import { ApiError } from './client'

/**
 * POSTs to an AI endpoint that answers with server-sent events: `progress` (characters
 * of the answer received so far) while Claude writes, then `done` with the result or
 * `error`. EventSource cannot POST or send the token, so the stream is read by hand.
 */
export async function streamAi<T>(path: string, body: unknown, onProgress: (chars: number) => void): Promise<T> {
  const token = useAuth.getState().token
  const res = await fetch('/api/v1' + path, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: body === undefined ? undefined : JSON.stringify(body),
  })
  if (res.status === 401) useAuth.getState().logout()
  if (!res.ok || !res.body) {
    const err = await res.json().catch(() => null)
    throw new ApiError(res.status, err?.error ?? res.statusText)
  }
  const reader = res.body.getReader()
  const decoder = new TextDecoder()
  let buf = ''
  for (;;) {
    const { value, done } = await reader.read()
    if (done) break
    buf += decoder.decode(value, { stream: true })
    let end: number
    while ((end = buf.indexOf('\n\n')) >= 0) {
      const raw = buf.slice(0, end)
      buf = buf.slice(end + 2)
      const event = /^event: ?(.*)$/m.exec(raw)?.[1] ?? 'message'
      const data = raw
        .split('\n')
        .filter((l) => l.startsWith('data:'))
        .map((l) => l.slice(5).replace(/^ /, ''))
        .join('\n')
      if (event === 'progress') onProgress(Number(data))
      else if (event === 'done') return JSON.parse(data) as T
      else if (event === 'error') throw new ApiError(409, JSON.parse(data).error)
    }
  }
  throw new ApiError(500, 'the AI answer stopped early — please try again')
}

/** Uploads one file as the raw request body, reporting progress (0–1). */
export function uploadFile<T>(path: string, file: Blob, name: string, onProgress: (p: number) => void): Promise<T> {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest()
    xhr.open('POST', '/api/v1' + path)
    const token = useAuth.getState().token
    if (token) xhr.setRequestHeader('Authorization', `Bearer ${token}`)
    xhr.setRequestHeader('X-File-Name', encodeURIComponent(name))
    xhr.upload.onprogress = (e) => e.lengthComputable && onProgress(e.loaded / e.total)
    xhr.onload = () => {
      let body: unknown = null
      try {
        body = JSON.parse(xhr.responseText)
      } catch {
        /* not JSON */
      }
      if (xhr.status === 401) useAuth.getState().logout()
      if (xhr.status >= 200 && xhr.status < 300) resolve(body as T)
      else
        reject(
          new ApiError(
            xhr.status,
            xhr.status === 413 ? 'the file is larger than 20 MB' : ((body as { error?: string })?.error ?? xhr.statusText),
          ),
        )
    }
    xhr.onerror = () => reject(new ApiError(0, 'upload failed — check the connection'))
    xhr.send(file)
  })
}

/** Fetches an authorised file and returns an object URL for viewing it. */
export async function fileUrl(path: string): Promise<string> {
  const token = useAuth.getState().token
  const res = await fetch('/api/v1' + path, { headers: token ? { Authorization: `Bearer ${token}` } : {} })
  if (!res.ok) throw new ApiError(res.status, res.statusText)
  return URL.createObjectURL(await res.blob())
}
