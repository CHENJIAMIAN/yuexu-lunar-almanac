// Chromium 自带 ICU 中国农历数据；离线运行且会随浏览器更新校准历法规则。
const LUNAR_MONTH_INDEX = Object.freeze({
  正月: 1, 二月: 2, 三月: 3, 四月: 4, 五月: 5, 六月: 6,
  七月: 7, 八月: 8, 九月: 9, 十月: 10, 冬月: 11, 十一月: 11, 腊月: 12, 十二月: 12
});
const LUNAR_DAY_NAMES = [
  '', '初一', '初二', '初三', '初四', '初五', '初六', '初七', '初八', '初九', '初十',
  '十一', '十二', '十三', '十四', '十五', '十六', '十七', '十八', '十九', '二十',
  '廿一', '廿二', '廿三', '廿四', '廿五', '廿六', '廿七', '廿八', '廿九', '三十'
];
const ANIMALS = { 子: '鼠', 丑: '牛', 寅: '虎', 卯: '兔', 辰: '龙', 巳: '蛇', 午: '马', 未: '羊', 申: '猴', 酉: '鸡', 戌: '狗', 亥: '猪' };
const lunarFormatter = new Intl.DateTimeFormat('zh-CN-u-ca-chinese', {
  year: 'numeric',
  month: 'long',
  day: 'numeric'
});

function normalizeDate(dateLike) {
  if (dateLike instanceof Date && !Number.isNaN(dateLike.getTime())) {
    return new Date(dateLike.getFullYear(), dateLike.getMonth(), dateLike.getDate(), 12);
  }
  if (typeof dateLike === 'string') {
    const match = dateLike.match(/^(\d{4})-(\d{1,2})-(\d{1,2})$/);
    if (match) {
      const date = new Date(Number(match[1]), Number(match[2]) - 1, Number(match[3]), 12);
      if (!Number.isNaN(date.getTime())) return date;
    }
  }
  throw new Error('Invalid date');
}

function ganzhiYear(year) {
  const stems = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸'];
  const branches = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'];
  const index = ((year - 4) % 60 + 60) % 60;
  const name = `${stems[index % 10]}${branches[index % 12]}`;
  return { name, animal: ANIMALS[name.at(-1)] };
}

function solarToLunar(dateLike) {
  const parts = lunarFormatter.formatToParts(normalizeDate(dateLike));
  const part = (type) => parts.find((item) => item.type === type)?.value;
  const lunarYear = Number(part('relatedYear'));
  const ganzhiName = part('yearName');
  const monthName = part('month');
  const day = Number(part('day'));
  const isLeap = monthName.startsWith('闰');
  const canonicalMonthName = isLeap ? monthName.slice(1) : monthName;
  const month = LUNAR_MONTH_INDEX[canonicalMonthName];

  if (!lunarYear || !month || !day) throw new Error('The browser does not provide usable Chinese lunar calendar data');
  return {
    year: lunarYear,
    month,
    day,
    isLeap,
    monthName,
    dayName: LUNAR_DAY_NAMES[day],
    ganzhi: { name: ganzhiName || ganzhiYear(lunarYear).name, animal: ANIMALS[(ganzhiName || ganzhiYear(lunarYear).name).at(-1)] }
  };
}

function lunarShortLabel(lunar) {
  return lunar.day === 1 ? lunar.monthName : lunar.dayName;
}

function lunarFullLabel(lunar) {
  return `${lunar.monthName}${lunar.dayName}`;
}

function lunarDataFor(dateLike) {
  return solarToLunar(dateLike);
}
