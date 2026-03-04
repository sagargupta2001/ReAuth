import type { ComponentType } from 'react'

import {
  AlertCircle,
  AlertTriangle,
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  ArrowUp,
  AtSign,
  BadgeCheck,
  Bell,
  Building,
  Calendar,
  Camera,
  Check,
  CheckCircle,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  ChevronUp,
  Clock,
  CreditCard,
  DollarSign,
  Download,
  Upload,
  Eye,
  EyeOff,
  ExternalLink,
  FileText,
  Globe,
  Grid,
  Heart,
  Home,
  Image,
  Info,
  Key,
  Link,
  Lock,
  LogIn,
  LogOut,
  Mail,
  MapPin,
  Minus,
  Moon,
  Phone,
  Plus,
  Search,
  Settings,
  Shield,
  Sparkles,
  Star,
  Sun,
  Trash,
  User,
  UserPlus,
  Users,
  X,
  XCircle,
  Zap,
} from 'lucide-react'

const ICON_REGISTRY: Record<string, ComponentType<{ size?: number; color?: string }>> = {
  mail: Mail,
  email: Mail,
  at: AtSign,
  user: User,
  profile: User,
  users: Users,
  userplus: UserPlus,
  adduser: UserPlus,
  login: LogIn,
  logout: LogOut,
  home: Home,
  house: Home,
  grid: Grid,
  file: FileText,
  doc: FileText,
  document: FileText,
  image: Image,
  camera: Camera,
  badge: BadgeCheck,
  verified: BadgeCheck,
  checkcircle: CheckCircle,
  check_circle: CheckCircle,
  xcircle: XCircle,
  x_circle: XCircle,
  warning: AlertTriangle,
  heart: Heart,
  star: Star,
  sparkles: Sparkles,
  zap: Zap,
  lock: Lock,
  key: Key,
  shield: Shield,
  settings: Settings,
  info: Info,
  check: Check,
  close: X,
  x: X,
  trash: Trash,
  phone: Phone,
  search: Search,
  link: Link,
  alert: AlertCircle,
  error: AlertCircle,
  alerttriangle: AlertTriangle,
  eye: Eye,
  eyeoff: EyeOff,
  eye_off: EyeOff,
  globe: Globe,
  calendar: Calendar,
  card: CreditCard,
  creditcard: CreditCard,
  dollar: DollarSign,
  money: DollarSign,
  download: Download,
  upload: Upload,
  externallink: ExternalLink,
  external: ExternalLink,
  arrowleft: ArrowLeft,
  arrowright: ArrowRight,
  arrowup: ArrowUp,
  arrowdown: ArrowDown,
  chevronup: ChevronUp,
  chevrondown: ChevronDown,
  building: Building,
  map: MapPin,
  location: MapPin,
  bell: Bell,
  sun: Sun,
  moon: Moon,
  chevronright: ChevronRight,
  chevronleft: ChevronLeft,
  plus: Plus,
  minus: Minus,
  clock: Clock,
}

export function renderIcon(
  name: string,
  props?: { size?: number; color?: string },
  options?: { svgPath?: string; viewBox?: string },
) {
  if (options?.svgPath) {
    const size = props?.size ?? 16
    const color = props?.color ?? 'currentColor'
    return (
      <svg
        width={size}
        height={size}
        viewBox={options.viewBox || '0 0 24 24'}
        fill="none"
        stroke={color}
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <path d={options.svgPath} />
      </svg>
    )
  }
  const Icon = ICON_REGISTRY[name.trim().toLowerCase()]
  if (!Icon) return null
  return <Icon size={props?.size} color={props?.color} />
}

export const ICON_NAMES = Object.keys(ICON_REGISTRY)
  .filter((name, index, arr) => arr.indexOf(name) === index)
  .sort()
