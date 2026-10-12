import { createBrowserRouter, Navigate } from 'react-router'
import { RequireAuth } from '../features/auth/RequireAuth'
import { LoginPage } from '../features/auth/LoginPage'
import { DashboardPage } from '../features/dashboard/DashboardPage'
import { PatientListPage } from '../features/patients/PatientListPage'
import { PatientProfilePage } from '../features/patients/PatientProfilePage'
import { PrescriptionPage } from '../features/prescription/PrescriptionPage'
import { AppointmentsPage } from '../features/appointments/AppointmentsPage'
import { FollowupsPage } from '../features/followups/FollowupsPage'
import { CollectionPage } from '../features/billing/CollectionPage'
import { AiUsagePage } from '../features/ai/AiUsagePage'
import { SettingsPage } from '../features/settings/SettingsPage'
import { Layout } from './Layout'

export const router = createBrowserRouter([
  { path: '/login', element: <LoginPage /> },
  {
    element: (
      <RequireAuth>
        <Layout />
      </RequireAuth>
    ),
    children: [
      { index: true, element: <Navigate to="/dashboard" replace /> },
      { path: '/dashboard', element: <DashboardPage /> },
      { path: '/patients', element: <PatientListPage /> },
      { path: '/patients/:id', element: <PatientProfilePage /> },
      { path: '/visits/:id/prescription', element: <PrescriptionPage /> },
      { path: '/appointments', element: <AppointmentsPage /> },
      { path: '/followups', element: <FollowupsPage /> },
      { path: '/collection', element: <CollectionPage /> },
      { path: '/ai-usage', element: <AiUsagePage /> },
      { path: '/settings', element: <SettingsPage /> },
    ],
  },
])
