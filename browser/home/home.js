/**
 * Aurexalis pagina de inicio: reloj, busqueda y accesos directos locales.
 */
(function () {
  "use strict";

  const STORAGE_KEY = "aurexalis.home.shortcuts.v1";

  const DEFAULT_SHORTCUTS = [
    {
      label: "GitHub",
      url: "https://github.com/JackStar6677-1/Aurexalis",
      color: "#6f38ff",
      letter: "G",
    },
    {
      label: "YouTube",
      url: "https://www.youtube.com/",
      color: "#ff1f55",
      letter: "Y",
    },
    {
      label: "Reddit",
      url: "https://www.reddit.com/",
      color: "#ff6314",
      letter: "R",
    },
    {
      label: "Wikipedia",
      url: "https://es.wikipedia.org/",
      color: "#b8a9cc",
      letter: "W",
    },
    {
      label: "DuckDuckGo",
      url: "https://duckduckgo.com/",
      color: "#ffd166",
      letter: "D",
    },
  ];

  const timeEl = document.getElementById("ax-time");
  const dateEl = document.getElementById("ax-date");
  const gridEl = document.getElementById("ax-shortcuts");
  const searchForm = document.getElementById("ax-search");
  const queryInput = document.getElementById("ax-query");
  const addDialog = document.getElementById("ax-add-dialog");
  const addForm = document.getElementById("ax-add-form");
  const addLabel = document.getElementById("ax-add-label");
  const addUrl = document.getElementById("ax-add-url");

  /** Actualiza reloj y fecha en es-ES. */
  function tickClock() {
    const now = new Date();
    timeEl.textContent = now.toLocaleTimeString("es-ES", {
      hour: "2-digit",
      minute: "2-digit",
      hour12: false,
    });
    dateEl.textContent = now
      .toLocaleDateString("es-ES", { weekday: "short", day: "2-digit", month: "2-digit" })
      .replace(",", "");
  }

  /** Carga accesos guardados o los valores por defecto. */
  function loadShortcuts() {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (!raw) {
        return DEFAULT_SHORTCUTS.slice();
      }
      const parsed = JSON.parse(raw);
      return Array.isArray(parsed) && parsed.length ? parsed : DEFAULT_SHORTCUTS.slice();
    } catch (_error) {
      return DEFAULT_SHORTCUTS.slice();
    }
  }

  /** Persiste accesos en localStorage del perfil. */
  function saveShortcuts(items) {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(items));
  }

  /** Crea un tile de acceso directo. */
  function createShortcutNode(item) {
    const li = document.createElement("li");
    const link = document.createElement("a");
    link.href = item.url;
    link.title = item.label;
    link.rel = "noopener noreferrer";

    const icon = document.createElement("span");
    icon.className = "tile-icon";
    icon.style.background = `linear-gradient(145deg, ${item.color || "#6f38ff"}, rgba(8,5,15,0.2))`;
    icon.textContent = item.letter || item.label.charAt(0).toUpperCase();

    const label = document.createElement("span");
    label.className = "tile-label";
    label.textContent = item.label;

    link.append(icon, label);
    li.append(link);
    return li;
  }

  /** Renderiza la cuadricula de accesos y el boton agregar. */
  function renderShortcuts(items) {
    gridEl.replaceChildren();
    for (const item of items) {
      gridEl.append(createShortcutNode(item));
    }

    const addLi = document.createElement("li");
    addLi.className = "tile-add";
    const addBtn = document.createElement("button");
    addBtn.type = "button";
    addBtn.title = "Agregar sitio";
    addBtn.innerHTML =
      '<span class="tile-icon">+</span><span class="tile-label">Agregar sitio</span>';
    addBtn.addEventListener("click", () => {
      addLabel.value = "";
      addUrl.value = "";
      addDialog.showModal();
    });
    addLi.append(addBtn);
    gridEl.append(addLi);
  }

  /** Detecta URLs sin esquema y las abre directamente. */
  function handleSearchSubmit(event) {
    const value = queryInput.value.trim();
    if (!value) {
      event.preventDefault();
      return;
    }

    const looksLikeUrl =
      /^(https?:\/\/|about:|chrome:|file:)/i.test(value) ||
      /^[\w.-]+\.[a-z]{2,}(\/.*)?$/i.test(value);

    if (looksLikeUrl) {
      event.preventDefault();
      const href = /^https?:\/\//i.test(value) ? value : `https://${value}`;
      window.location.href = href;
    }
  }

  addForm.addEventListener("submit", (event) => {
    event.preventDefault();
    const label = addLabel.value.trim();
    const url = addUrl.value.trim();
    if (!label || !url) {
      return;
    }

    const items = loadShortcuts();
    items.push({
      label,
      url,
      color: "#6f38ff",
      letter: label.charAt(0).toUpperCase(),
    });
    saveShortcuts(items);
    renderShortcuts(items);
    addDialog.close();
  });

  addForm.addEventListener("reset", () => {
    addDialog.close();
  });

  searchForm.addEventListener("submit", handleSearchSubmit);
  queryInput.focus();

  tickClock();
  setInterval(tickClock, 30_000);
  renderShortcuts(loadShortcuts());
})();
