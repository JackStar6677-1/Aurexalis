/**
 * Ajustes Aurexalis — pagina interactiva (desktop via AurexalisPrefsBridge, Android via aurexalis://).
 */
(function () {
  "use strict";

  const logEl = document.getElementById("import-log");
  const params = new URLSearchParams(window.location.search);
  const isAndroid = /Android/i.test(navigator.userAgent);

  /** Espera al puente inyectado por chrome (desktop) o usa beacon Android. */
  function withBridge(callback) {
    if (window.AurexalisPrefsBridge) {
      callback(window.AurexalisPrefsBridge);
      return;
    }
    window.addEventListener(
      "aurexalis-prefs-ready",
      () => callback(window.AurexalisPrefsBridge),
      { once: true }
    );
    setTimeout(() => {
      if (window.AurexalisPrefsBridge) {
        callback(window.AurexalisPrefsBridge);
      } else {
        callback(createAndroidBridge());
      }
    }, 400);
  }

  /** Puente minimo para Android (MainActivity intercepta aurexalis://pref/). */
  function createAndroidBridge() {
    function beacon(type, name, value) {
      const img = new Image();
      img.src =
        "aurexalis://pref/set?type=" +
        encodeURIComponent(type) +
        "&name=" +
        encodeURIComponent(name) +
        "&value=" +
        encodeURIComponent(String(value));
    }
    return {
      getBool(name, fallback) {
        const v = params.get("b." + name);
        return v === null ? fallback : v === "1" || v === "true";
      },
      setBool(name, value) {
        beacon("bool", name, value ? "1" : "0");
      },
      getInt(name, fallback) {
        const v = params.get("i." + name);
        return v === null ? fallback : Number(v);
      },
      setInt(name, value) {
        beacon("int", name, value);
      },
      getString(name, fallback) {
        const v = params.get("s." + name);
        return v === null ? fallback : v;
      },
      setString(name, value) {
        beacon("string", name, value);
      },
    };
  }

  function showLog(message) {
    logEl.hidden = false;
    logEl.textContent = message;
  }

  function bindToggle(id, pref, fallback) {
    const el = document.getElementById(id);
    if (!el) return;
    withBridge((bridge) => {
      el.checked = bridge.getBool(pref, fallback);
      el.addEventListener("change", () => bridge.setBool(pref, el.checked));
    });
  }

  function bindRange(id, pref, fallback, labelId) {
    const el = document.getElementById(id);
    if (!el) return;
    withBridge((bridge) => {
      const val = bridge.getInt(pref, fallback);
      el.value = String(val);
      if (labelId) {
        const label = document.getElementById(labelId);
        if (label) label.textContent = val + "%";
      }
      el.addEventListener("input", () => {
        const n = Number(el.value);
        bridge.setInt(pref, n);
        if (labelId) {
          const label = document.getElementById(labelId);
          if (label) label.textContent = n + "%";
        }
      });
    });
  }

  function bindSelect(id, pref, fallback) {
    const el = document.getElementById(id);
    if (!el) return;
    withBridge((bridge) => {
      el.value = bridge.getString(pref, fallback);
      el.addEventListener("change", () => bridge.setString(pref, el.value));
    });
  }

  function importHint(withPasswords) {
    if (isAndroid) {
      showLog(
        withPasswords
          ? "Importacion Chromium + contrasenas: disponible en Aurexalis PC."
          : "Navegacion y home Aurexalis activas. Importacion completa en escritorio."
      );
      return;
    }
    const cmd = withPasswords
      ? "aurexalis import audit --passwords"
      : "aurexalis import audit";
    showLog(
      "Desde la carpeta de instalacion ejecuta:\n\n  " +
        cmd +
        "\n\nO usa el panel ST del sidebar.\n\nSalida: profiles\\default\\import\\chromium-audit.json"
    );
  }

  bindToggle("pref-sounds-enabled", "sounds.enabled", true);
  bindRange("pref-sounds-master", "sounds.master", 22, "val-sounds-master");
  bindToggle("pref-sounds-click", "sounds.click.enabled", true);
  bindRange("pref-vol-click", "sounds.click.volume", 85, "val-vol-click");
  bindToggle("pref-sounds-hover", "sounds.hover.enabled", true);
  bindRange("pref-vol-hover", "sounds.hover.volume", 55, "val-vol-hover");
  bindToggle("pref-sounds-key", "sounds.key.enabled", true);
  bindRange("pref-vol-key", "sounds.key.volume", 40, "val-vol-key");
  bindToggle("pref-sounds-ambient", "sounds.ambient.enabled", true);
  bindRange("pref-vol-ambient", "sounds.ambient.volume", 12, "val-vol-ambient");
  bindToggle("pref-sounds-panel", "sounds.panel.enabled", true);
  bindRange("pref-vol-panel", "sounds.panel.volume", 70, "val-vol-panel");
  bindToggle("pref-ui-animations", "ui.animations", true);

  bindToggle("pref-blocker-enabled", "blocker.enabled", true);
  bindSelect("pref-blocker-level", "blocker.level", "standard");
  bindToggle("pref-blocker-cosmetic", "blocker.cosmetic", true);

  bindToggle("pref-cws-enabled", "cws.enabled", true);
  bindToggle("pref-cws-brand", "cws.brandPrompts", true);

  document.getElementById("btn-open-addons")?.addEventListener("click", () => {
    showLog("Abre about:addons en una pestaña nueva, o usa el boton EX del sidebar.");
  });
  document.getElementById("btn-open-cws")?.addEventListener("click", () => {
    withBridge((bridge) => {
      const url = bridge.getString("cws.storeUrl", "https://chromewebstore.google.com/");
      showLog("Abre Chrome Web Store:\n\n  " + url + "\n\nSidebar EX abre gestor + tienda.");
    });
  });

  document.getElementById("btn-import-data")?.addEventListener("click", () => importHint(false));
  document.getElementById("btn-import-passwords")?.addEventListener("click", () => {
    if (window.confirm("Se exportaran contrasenas a un JSON local. ¿Continuar?")) {
      importHint(true);
    }
  });

  document.getElementById("btn-import-apply")?.addEventListener("click", () => {
    if (
      window.confirm(
        "Cierra el navegador antes de aplicar. ¿Ejecutar import apply (marcadores + historial)?"
      )
    ) {
      showLog(
        "Ejecuta desde la carpeta de instalacion:\n\n  aurexalis import apply\n\nO usa el panel IM/ST."
      );
    }
  });

  document.getElementById("btn-blocker-sync")?.addEventListener("click", () => {
    showLog("Ejecuta:\n\n  aurexalis blocker sync-lists\n\nO boton ST → Sincronizar listas bloqueador.");
  });

  if (params.get("import") === "passwords") importHint(true);
  else if (params.get("import") === "data") importHint(false);
})();
