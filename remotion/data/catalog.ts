export interface Category {
  name: string;
  count: number;
  icon: string;
}

export const CATEGORIES: Category[] = [
  { name: "infrastructure", count: 6, icon: "⚙️" },
  { name: "language", count: 5, icon: "💻" },
  { name: "security", count: 4, icon: "🔒" },
  { name: "database", count: 5, icon: "🗄️" },
  { name: "editor", count: 3, icon: "✏️" },
  { name: "terminal", count: 4, icon: "⌨️" },
  { name: "networking", count: 4, icon: "🌐" },
  { name: "devops", count: 3, icon: "🚀" },
  { name: "monitoring", count: 3, icon: "📊" },
  { name: "testing", count: 3, icon: "🧪" },
  { name: "utilities", count: 2, icon: "🔧" },
];

export const TOTAL_PACKAGES = 42;
export const TOTAL_CATEGORIES = 11;
export const TOTAL_ALIASES = 13;
