// Time utility functions

export const formatTimeAgo = (timestamp, t) => {
  if (!timestamp) return t ? t('time.never') : 'Never'
  
  const now = new Date()
  const past = new Date(timestamp)
  const diffMs = now - past
  
  // If the timestamp is in the future (negative diff), show "Just now"
  if (diffMs < 0) return t ? t('time.just_now') : 'Just now'
  
  const seconds = Math.floor(diffMs / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  
  if (t) {
    if (days > 0) {
      return t('time.days_ago', { count: days })
    } else if (hours > 0) {
      return t('time.hours_ago', { count: hours })
    } else if (minutes > 0) {
      return t('time.minutes_ago', { count: minutes })
    } else if (seconds > 10) {
      return t('time.seconds_ago', { count: seconds })
    } else {
      return t('time.just_now')
    }
  }

  if (days > 0) {
    return `${days}d ${hours % 24}h ago`
  } else if (hours > 0) {
    return `${hours}h ${minutes % 60}m ago`
  } else if (minutes > 0) {
    return `${minutes}m ago`
  } else if (seconds > 10) {
    return `${seconds}s ago`
  } else {
    return 'Just now'
  }
}

export const formatDuration = (hours) => {
  if (!hours || hours <= 0) return '0m'
  
  if (hours < 1) {
    const minutes = Math.round(hours * 60)
    return `${minutes}m`
  }
  
  const h = Math.floor(hours)
  const m = Math.round((hours - h) * 60)
  
  if (h >= 24) {
    const days = Math.floor(h / 24)
    const remainingHours = h % 24
    if (remainingHours === 0) {
      return `${days}d`
    }
    return `${days}d ${remainingHours}h`
  }
  
  return m > 0 ? `${h}h ${m}m` : `${h}h`
}

export const formatUptime = (startTime) => {
  if (!startTime) return '0m'
  
  const now = new Date()
  const start = new Date(startTime)
  const diffMs = now - start
  
  if (diffMs < 0) return '0m'
  
  const hours = diffMs / (1000 * 60 * 60)
  return formatDuration(hours)
}