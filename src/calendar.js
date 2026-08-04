const MONTH_NAMES = ['一月', '二月', '三月', '四月', '五月', '六月', '七月', '八月', '九月', '十月', '十一月', '十二月'];
const MONTH_EN = ['JANUARY', 'FEBRUARY', 'MARCH', 'APRIL', 'MAY', 'JUNE', 'JULY', 'AUGUST', 'SEPTEMBER', 'OCTOBER', 'NOVEMBER', 'DECEMBER'];
const WEEK_NAMES = ['一', '二', '三', '四', '五', '六', '日'];
const SEASON_NAMES = ['冬 · Winter', '冬 · Winter', '春 · Spring', '春 · Spring', '春 · Spring', '夏 · Summer', '夏 · Summer', '夏 · Summer', '秋 · Autumn', '秋 · Autumn', '秋 · Autumn', '冬 · Winter'];

function pad2(value) {
  return String(value).padStart(2, '0');
}

function dateKey(year, month, day) {
  return `${year}-${pad2(month + 1)}-${pad2(day)}`;
}

function localTodayKey() {
  const now = new Date();
  return dateKey(now.getFullYear(), now.getMonth(), now.getDate());
}

function parseQuery() {
  const query = new URLSearchParams(window.location.search);
  const year = Number(query.get('year')) || new Date().getFullYear();
  const today = query.get('today') || localTodayKey();
  const wallpaper = query.get('wallpaper') === '1';
  const theme = query.get('theme') === 'light' ? 'light' : 'dark';
  const width = Number(query.get('width')) || 1920;
  const height = Number(query.get('height')) || 1080;
  return { year: Math.min(2100, Math.max(1900, year)), today, wallpaper, theme, width, height };
}

function createYearData(year, todayKey) {
  const months = [];
  for (let month = 0; month < 12; month += 1) {
    const daysInMonth = new Date(Date.UTC(year, month + 1, 0)).getUTCDate();
    const firstWeekday = new Date(Date.UTC(year, month, 1)).getUTCDay();
    const mondayOffset = (firstWeekday + 6) % 7;
    const days = [];
    for (let i = 0; i < mondayOffset; i += 1) days.push(null);
    for (let day = 1; day <= daysInMonth; day += 1) {
      const key = dateKey(year, month, day);
      const lunar = lunarDataFor(key);
      days.push({ day, key, lunar, isToday: key === todayKey, isWeekend: (days.length - mondayOffset) % 7 >= 5 });
    }
    while (days.length < 42) days.push(null);
    months.push({ month, name: MONTH_NAMES[month], english: MONTH_EN[month], season: SEASON_NAMES[month], days });
  }
  return months;
}

function getYearMeta(year, todayKey) {
  const focus = todayKey.startsWith(`${year}-`) ? todayKey : `${year}-06-15`;
  const lunar = lunarDataFor(focus);
  const month = Number(focus.slice(5, 7)) - 1;
  const day = Number(focus.slice(8, 10));
  const date = new Date(Date.UTC(year, month, day));
  return {
    focus,
    date,
    day,
    month,
    lunar,
    isCurrentYear: todayKey.startsWith(`${year}-`),
    weekday: WEEK_NAMES[(date.getUTCDay() + 6) % 7]
  };
}
