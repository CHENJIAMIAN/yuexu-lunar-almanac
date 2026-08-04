(function bootstrap() {
  const state = parseQuery();
  const yearData = createYearData(state.year, state.today);
  const meta = getYearMeta(state.year, state.today);
  const root = document.documentElement;
  const body = document.body;

  body.classList.toggle('wallpaper-mode', state.wallpaper);
  body.dataset.theme = state.theme;
  root.style.setProperty('--canvas-width', `${state.width}px`);
  root.style.setProperty('--canvas-height', `${state.height}px`);
  root.style.setProperty('--canvas-scale', String(Math.min(state.width / 1920, state.height / 1080)));
  document.title = `月序 · ${state.year}`;

  const setText = (selector, value) => {
    const node = document.querySelector(selector);
    if (node) node.textContent = value;
  };
  setText('[data-year]', state.year);
  setText('[data-year-cn]', ` · ${meta.lunar.ganzhi.name}年`);

  const monthGrid = document.querySelector('[data-month-grid]');
  monthGrid.innerHTML = yearData.map((monthInfo) => {
    const isCurrentMonth = meta.isCurrentYear && monthInfo.month === meta.month;
    const dayMarkup = monthInfo.days.map((entry) => {
      if (!entry) return '<span class="day-cell empty" aria-hidden="true"></span>';
      const classes = ['day-cell'];
      if (entry.isWeekend) classes.push('weekend');
      if (entry.isToday) classes.push('today');
      return `<span class="${classes.join(' ')}" data-date="${entry.key}"><b>${entry.day}</b><small>${lunarShortLabel(entry.lunar)}</small></span>`;
    }).join('');
    return `<article class="month-card${isCurrentMonth ? ' current-month' : ''}">
      <header class="month-heading"><div><span class="month-index">${pad2(monthInfo.month + 1)}</span><strong>${monthInfo.name}</strong></div><span class="month-en">${monthInfo.english}</span></header>
      <div class="week-row">${WEEK_NAMES.map((name) => `<span>${name}</span>`).join('')}</div>
      <div class="days-grid">${dayMarkup}</div>
    </article>`;
  }).join('');

  const yearControl = document.querySelector('[data-year-control]');
  yearControl.textContent = state.year;
  document.querySelector('[data-prev-year]').addEventListener('click', () => navigateYear(state.year - 1));
  document.querySelector('[data-next-year]').addEventListener('click', () => navigateYear(state.year + 1));
  document.querySelectorAll('[data-theme-choice]').forEach((button) => {
    button.classList.toggle('selected', button.dataset.themeChoice === state.theme);
    button.addEventListener('click', () => {
      if (new URLSearchParams(window.location.search).get('native') === '1') {
        window.location.href = `yuexu://theme/${button.dataset.themeChoice}`;
        return;
      }
      navigateTheme(button.dataset.themeChoice);
    });
  });
  document.querySelector('[data-open-wallpaper]').addEventListener('click', () => {
    const url = new URL(window.location.href);
    url.searchParams.set('wallpaper', '1');
    url.searchParams.set('width', '1920');
    url.searchParams.set('height', '1080');
    window.open(url.toString(), '_blank', 'noopener');
  });

  function navigateYear(nextYear) {
    const url = new URL(window.location.href);
    url.searchParams.set('year', String(Math.min(2100, Math.max(1900, nextYear))));
    window.location.href = url.toString();
  }

  function navigateTheme(nextTheme) {
    const url = new URL(window.location.href);
    url.searchParams.set('theme', nextTheme);
    window.location.href = url.toString();
  }

  // 让壁纸模式只保留一个固定画布，浏览器不会因为窗口尺寸改变排版。
  if (state.wallpaper) {
    document.body.style.width = `${state.width}px`;
    document.body.style.height = `${state.height}px`;
  }
})();
