import { useState } from 'react'
import { Bot, Package, BookOpen } from 'lucide-react'
import { cn } from '@/lib/utils'
import { BotsPage } from '@/pages/BotsPage'
import { ItemsPage } from '@/pages/ItemsPage'
import { DocsPage } from '@/pages/DocsPage'

type Page = 'bots' | 'items' | 'docs'

export default function App() {
  const [page, setPage] = useState<Page>('bots')

  return (
    <div className="h-full flex flex-col antialiased text-sm">
      <header className="shrink-0 h-10 flex items-center justify-between px-4 bg-card border-b border-border">
        <span className="font-bold tracking-wide text-foreground text-xs">
          Mori<span className="text-primary">.</span>
        </span>
        <nav className="flex gap-0.5">
          <NavBtn active={page === 'bots'} onClick={() => setPage('bots')} icon={<Bot className="w-3 h-3" />} label="Bots" />
          <NavBtn active={page === 'items'} onClick={() => setPage('items')} icon={<Package className="w-3 h-3" />} label="Items" />
          <NavBtn active={page === 'docs'} onClick={() => setPage('docs')} icon={<BookOpen className="w-3 h-3" />} label="Docs" />
        </nav>
      </header>
      <div className="flex-1 overflow-hidden">
        {page === 'bots' && <BotsPage />}
        {page === 'items' && <ItemsPage />}
        {page === 'docs' && <DocsPage />}
      </div>
    </div>
  )
}

function NavBtn({
  active,
  onClick,
  icon,
  label,
}: {
  active: boolean
  onClick: () => void
  icon: React.ReactNode
  label: string
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        'px-3 py-1 text-xs rounded font-medium flex items-center gap-1.5 transition-colors',
        active ? 'text-primary bg-muted' : 'text-muted-foreground hover:text-foreground',
      )}
    >
      {icon}
      {label}
    </button>
  )
}
