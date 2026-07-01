import { NavLink } from 'react-router-dom'

const links = [
  { to: '/', label: 'Dashboard', icon: '⬡' },
  { to: '/activities', label: 'Activities', icon: '⚡' },
  { to: '/wellness', label: 'Wellness', icon: '♥' },
]

export function Nav() {
  return (
    <nav className="w-56 flex-shrink-0 bg-slate-800 border-r border-slate-700 flex flex-col min-h-screen">
      <div className="px-6 py-5 border-b border-slate-700">
        <div className="text-indigo-400 font-bold text-lg tracking-wide">GARMIN</div>
        <div className="text-slate-400 text-xs mt-0.5">Dashboard</div>
      </div>
      <div className="flex flex-col gap-1 p-3 flex-1">
        {links.map((l) => (
          <NavLink
            key={l.to}
            to={l.to}
            end={l.to === '/'}
            className={({ isActive }) =>
              `flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors
               ${isActive
                ? 'bg-indigo-600 text-white'
                : 'text-slate-400 hover:text-slate-200 hover:bg-slate-700'
              }`
            }
          >
            <span className="text-base">{l.icon}</span>
            {l.label}
          </NavLink>
        ))}
      </div>
      <div className="px-4 pb-4 text-xs text-slate-600 text-center">
        garmin-dash
      </div>
    </nav>
  )
}
