import { MapContainer, TileLayer, Polyline, CircleMarker, Tooltip } from 'react-leaflet'
import 'leaflet/dist/leaflet.css'
import type { ActivityRecord } from '../types'

interface ActivityMapProps {
  records: ActivityRecord[]
}

export function ActivityMap({ records }: ActivityMapProps) {
  const points = records
    .filter((r) => r.lat !== null && r.lon !== null)
    .map((r) => [r.lat as number, r.lon as number] as [number, number])

  if (points.length === 0) return null

  const lats = points.map((p) => p[0])
  const lons = points.map((p) => p[1])
  const bounds: [[number, number], [number, number]] = [
    [Math.min(...lats), Math.min(...lons)],
    [Math.max(...lats), Math.max(...lons)],
  ]

  return (
    <div className="bg-slate-800 rounded-xl p-4">
      <h2 className="text-xs font-semibold text-slate-400 uppercase tracking-wide mb-3">Route</h2>
      <div className="rounded-lg overflow-hidden" style={{ height: 320 }}>
        <MapContainer
          bounds={bounds}
          boundsOptions={{ padding: [20, 20] }}
          style={{ height: '100%', width: '100%' }}
          scrollWheelZoom={true}
        >
          <TileLayer
            attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors &copy; <a href="https://carto.com/attributions">CARTO</a>'
            url="https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png"
          />
          <Polyline positions={points} pathOptions={{ color: '#6366f1', weight: 3 }} />
          <CircleMarker
            center={points[0]}
            radius={5}
            pathOptions={{ color: '#22c55e', fillColor: '#22c55e', fillOpacity: 1 }}
          >
            <Tooltip>Start</Tooltip>
          </CircleMarker>
          <CircleMarker
            center={points[points.length - 1]}
            radius={5}
            pathOptions={{ color: '#f43f5e', fillColor: '#f43f5e', fillOpacity: 1 }}
          >
            <Tooltip>Finish</Tooltip>
          </CircleMarker>
        </MapContainer>
      </div>
    </div>
  )
}
