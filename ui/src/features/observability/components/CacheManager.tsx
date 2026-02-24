import { useMemo, useState } from 'react'

import { AlertTriangle, Database, MemoryStick, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger } from '@/components/alert-dialog'
import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/card'
import { Input } from '@/components/input'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/table'

import { useCacheFlush } from '../api/useCacheFlush'
import { useCacheStats } from '../api/useCacheStats'

function formatPercent(value: number) {
  if (!Number.isFinite(value)) return '0%'
  return `${(value * 100).toFixed(1)}%`
}

export function CacheManager() {
  const { t } = useTranslation('logs')
  const { data, isLoading } = useCacheStats()
  const stats = Array.isArray(data) ? data : data ? [data] : []
  const cacheFlush = useCacheFlush()
  const [confirmOpen, setConfirmOpen] = useState(false)
  const [confirmInput, setConfirmInput] = useState('')

  const totals = useMemo(() => {
    const totalEntries = stats.reduce((sum, item) => sum + item.entry_count, 0)
    const totalCapacity = stats.reduce((sum, item) => sum + item.max_capacity, 0)
    const weightedHitRate =
      totalEntries === 0
        ? 0
        : stats.reduce((sum, item) => sum + item.hit_rate * item.entry_count, 0) / totalEntries
    return { totalEntries, totalCapacity, weightedHitRate }
  }, [stats])

  const usagePercent = totals.totalCapacity
    ? totals.totalEntries / totals.totalCapacity
    : 0

  return (
    <div className="flex h-full flex-col gap-4">
      <div className="grid gap-4 md:grid-cols-3">
        <Card className="border-emerald-500/30 bg-emerald-500/5">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm text-muted-foreground">
              {t('CACHE_MANAGER.OVERALL_HIT_RATE')}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-semibold">{formatPercent(totals.weightedHitRate)}</div>
          </CardContent>
        </Card>
        <Card className="border-sky-500/30 bg-sky-500/5">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm text-muted-foreground">
              {t('CACHE_MANAGER.TOTAL_ITEMS')}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-semibold">{totals.totalEntries}</div>
          </CardContent>
        </Card>
        <Card className="border-amber-500/30 bg-amber-500/5">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm text-muted-foreground">
              {t('CACHE_MANAGER.MEMORY_USAGE')}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-semibold">{formatPercent(usagePercent)}</div>
            <p className="text-xs text-muted-foreground">
              {totals.totalEntries} / {totals.totalCapacity}
            </p>
          </CardContent>
        </Card>
      </div>

      <div className="rounded-xl border bg-background/40">
        <div className="flex items-center justify-between border-b px-4 py-3">
          <div>
            <div className="text-sm font-semibold">{t('CACHE_MANAGER.NAMESPACE_TABLE')}</div>
            <p className="text-xs text-muted-foreground">{t('CACHE_MANAGER.NAMESPACE_SUBTITLE')}</p>
          </div>
          <Badge variant="secondary" className="gap-1">
            <Database className="h-3 w-3" />
            {stats.length} namespaces
          </Badge>
        </div>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t('CACHE_MANAGER.TABLE.NAMESPACE')}</TableHead>
                <TableHead>{t('CACHE_MANAGER.TABLE.ITEMS')}</TableHead>
                <TableHead>{t('CACHE_MANAGER.TABLE.HIT_RATE')}</TableHead>
                <TableHead>{t('CACHE_MANAGER.TABLE.CAPACITY')}</TableHead>
                <TableHead className="text-right">{t('CACHE_MANAGER.TABLE.ACTIONS')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading && stats.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={5} className="py-8 text-center text-muted-foreground">
                    {t('CACHE_MANAGER.LOADING')}
                  </TableCell>
                </TableRow>
              ) : stats.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={5} className="py-8 text-center text-muted-foreground">
                    {t('CACHE_MANAGER.EMPTY')}
                  </TableCell>
                </TableRow>
              ) : (
                stats.map((stat) => (
                  <TableRow key={stat.namespace}>
                    <TableCell className="font-mono text-xs">{stat.namespace}</TableCell>
                    <TableCell>{stat.entry_count}</TableCell>
                    <TableCell>{formatPercent(stat.hit_rate)}</TableCell>
                    <TableCell>
                      {stat.entry_count} / {stat.max_capacity}
                    </TableCell>
                    <TableCell className="text-right">
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => cacheFlush.mutate(stat.namespace)}
                        disabled={cacheFlush.isPending}
                        className="gap-1"
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                        {t('CACHE_MANAGER.PURGE_ACTION')}
                      </Button>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </div>
      </div>

      <div
        id="cache-danger-zone"
        className="rounded-xl border border-destructive/50 bg-destructive/10 p-4"
      >
        <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div className="flex items-start gap-3">
            <div className="rounded-full bg-destructive/20 p-2 text-destructive">
              <AlertTriangle className="h-4 w-4" />
            </div>
            <div>
              <div className="text-sm font-semibold text-destructive">
                {t('CACHE_MANAGER.DANGER_TITLE')}
              </div>
              <p className="text-xs text-muted-foreground">{t('CACHE_MANAGER.DANGER_DESC')}</p>
            </div>
          </div>
          <AlertDialog
            open={confirmOpen}
            onOpenChange={(open) => {
              setConfirmOpen(open)
              if (!open) setConfirmInput('')
            }}
          >
            <AlertDialogTrigger asChild>
              <Button variant="destructive" className="gap-2">
                <MemoryStick className="h-4 w-4" />
                {t('CACHE_MANAGER.FLUSH_ALL')}
              </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>{t('CACHE_MANAGER.CONFIRM_TITLE')}</AlertDialogTitle>
                <AlertDialogDescription>
                  {t('CACHE_MANAGER.CONFIRM_DESC')}
                </AlertDialogDescription>
              </AlertDialogHeader>
              <div className="space-y-2">
                <Input
                  placeholder={t('CACHE_MANAGER.CONFIRM_PLACEHOLDER')}
                  value={confirmInput}
                  onChange={(event) => setConfirmInput(event.target.value)}
                />
                <p className="text-xs text-muted-foreground">
                  {t('CACHE_MANAGER.CONFIRM_HELPER')}
                </p>
              </div>
              <AlertDialogFooter>
                <AlertDialogCancel>{t('CACHE_MANAGER.CANCEL')}</AlertDialogCancel>
                <AlertDialogAction
                  className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                  onClick={() => {
                    cacheFlush.mutate(undefined)
                    setConfirmInput('')
                  }}
                  disabled={confirmInput.trim() !== 'CONFIRM' || cacheFlush.isPending}
                >
                  {t('CACHE_MANAGER.CONFIRM_ACTION')}
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </div>
      </div>
    </div>
  )
}
