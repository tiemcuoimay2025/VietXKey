(() => {
  const root = document.documentElement;
  const systemTheme = matchMedia('(prefers-color-scheme: light)');
  const setTheme = (theme, persist = true) => {
    root.dataset.theme = theme;
    if (persist) { try { localStorage.setItem('vietxkey-theme', theme); } catch {} }
    const action = theme === 'dark' ? 'Nền sáng' : 'Nền tối';
    document.querySelector('.theme-label').textContent = action;
    document.querySelector('.theme-toggle').setAttribute('aria-label', `Chuyển sang ${action.toLowerCase()}`);
    document.querySelectorAll('[data-set-theme]').forEach(button => button.setAttribute('aria-pressed', String(button.dataset.setTheme === theme)));
    for (const id of ['app-screenshot', 'expanded-screenshot']) document.getElementById(id).src = `assets/overview-${theme}.png`;
    document.querySelector('meta[name="theme-color"]').content = theme === 'dark' ? '#17191d' : '#e9e9e6';
  };
  setTheme(root.dataset.theme, false);
  document.querySelector('.theme-toggle').addEventListener('click', () => setTheme(root.dataset.theme === 'dark' ? 'light' : 'dark'));
  document.querySelectorAll('[data-set-theme]').forEach(button => button.addEventListener('click', () => setTheme(button.dataset.setTheme)));
  systemTheme.addEventListener('change', event => {
    try { if (localStorage.getItem('vietxkey-theme')) return; } catch {}
    setTheme(event.matches ? 'light' : 'dark', false);
  });
  const support = document.getElementById('support-dialog');
  const preview = document.getElementById('preview-dialog');
  document.querySelector('[data-open-support]').addEventListener('click', () => support.showModal());
  document.querySelector('[data-open-preview]').addEventListener('click', () => preview.showModal());
  document.querySelectorAll('dialog').forEach(dialog => {
    dialog.querySelectorAll('[data-close-dialog]').forEach(button => button.addEventListener('click', () => dialog.close()));
    let backdropPress = false;
    const outside = event => {
      const rect = dialog.getBoundingClientRect();
      return event.clientX < rect.left || event.clientX > rect.right || event.clientY < rect.top || event.clientY > rect.bottom;
    };
    dialog.addEventListener('pointerdown', event => { backdropPress = event.target === dialog && outside(event); });
    dialog.addEventListener('click', event => { if (backdropPress && event.target === dialog && outside(event)) dialog.close(); backdropPress = false; });
    dialog.addEventListener('close', () => { backdropPress = false; });
  });
})();
