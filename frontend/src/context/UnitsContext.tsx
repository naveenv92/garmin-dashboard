import { createContext, useContext, useEffect, useState } from 'react'
import type { ReactNode } from 'react'

export type UnitSystem = 'metric' | 'imperial'

const STORAGE_KEY = 'garmin-dash-units'
const METERS_PER_MILE = 1609.344
const METERS_PER_FOOT = 0.3048

interface UnitsContextValue {
  units: UnitSystem
  setUnits: (u: UnitSystem) => void
  distanceUnit: string
  elevationUnit: string
  formatDistance: (meters: number | null | undefined) => string
  formatDistanceKm: (km: number) => string
  formatElevation: (meters: number | null | undefined) => string
  formatPace: (speedMs: number | null | undefined) => string
}

const UnitsContext = createContext<UnitsContextValue | null>(null)

export function UnitsProvider({ children }: { children: ReactNode }) {
  const [units, setUnits] = useState<UnitSystem>(() => {
    return localStorage.getItem(STORAGE_KEY) === 'imperial' ? 'imperial' : 'metric'
  })

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, units)
  }, [units])

  const distanceUnit = units === 'imperial' ? 'mi' : 'km'
  const elevationUnit = units === 'imperial' ? 'ft' : 'm'

  const formatDistance = (meters: number | null | undefined) => {
    if (!meters) return '—'
    const value = units === 'imperial' ? meters / METERS_PER_MILE : meters / 1000
    return value.toFixed(2) + ' ' + distanceUnit
  }

  const formatDistanceKm = (km: number) => {
    const value = units === 'imperial' ? km / 1.609344 : km
    return value.toFixed(0) + ' ' + distanceUnit
  }

  const formatElevation = (meters: number | null | undefined) => {
    if (!meters) return '—'
    const value = units === 'imperial' ? meters / METERS_PER_FOOT : meters
    return Math.round(value).toLocaleString() + ' ' + elevationUnit
  }

  const formatPace = (speedMs: number | null | undefined) => {
    if (!speedMs || speedMs === 0) return '—'
    const unitMeters = units === 'imperial' ? METERS_PER_MILE : 1000
    const minPerUnit = unitMeters / 60 / speedMs
    const min = Math.floor(minPerUnit)
    const sec = Math.round((minPerUnit - min) * 60)
    return `${min}:${sec.toString().padStart(2, '0')} /${distanceUnit}`
  }

  return (
    <UnitsContext.Provider
      value={{
        units,
        setUnits,
        distanceUnit,
        elevationUnit,
        formatDistance,
        formatDistanceKm,
        formatElevation,
        formatPace,
      }}
    >
      {children}
    </UnitsContext.Provider>
  )
}

export function useUnits() {
  const ctx = useContext(UnitsContext)
  if (!ctx) throw new Error('useUnits must be used within a UnitsProvider')
  return ctx
}
