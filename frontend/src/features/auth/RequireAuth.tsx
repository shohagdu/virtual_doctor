import type { ReactNode } from 'react'
import { Navigate } from 'react-router'
import { useAuth } from './authStore'

export function RequireAuth({ children }: { children: ReactNode }) {
  const token = useAuth((s) => s.token)
  if (!token) return <Navigate to="/login" replace />
  return children
}
