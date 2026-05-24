// Nucleo compartido Aurexalis: preferencias, lanzador y utilidades de chrome.

(function initAurexalisCore() {
  "use strict";

  if (window.AurexalisCore) {
    return;
  }

  const PREF_ROOT = "aurexalis.";

  /** Lee un booleano de preferencias con valor por defecto. */
  function getBool(name, fallback) {
    try {
      return Services.prefs.getBoolPref(PREF_ROOT + name, fallback);
    } catch (_error) {
      return fallback;
    }
  }

  /** Escribe un booleano en preferencias de usuario. */
  function setBool(name, value) {
    try {
      Services.prefs.setBoolPref(PREF_ROOT + name, value);
    } catch (error) {
      console.warn("[AurexalisCore] setBool", name, error);
    }
    if (name.startsWith("blocker.") && window.AurexalisBlocker) {
      AurexalisBlocker.applyBlockerPrefs();
    }
    if (name.startsWith("sounds.") && window.AurexalisSound) {
      AurexalisSound.refreshAmbient();
    }
    if (name === "ui.animations") {
      document.documentElement.classList.toggle("ax-animations-off", !value);
    }
  }

  /** Lee un entero 0..100 desde preferencias. */
  function getInt(name, fallback) {
    try {
      return Services.prefs.getIntPref(PREF_ROOT + name, fallback);
    } catch (_error) {
      return fallback;
    }
  }

  /** Guarda un entero en preferencias. */
  function setInt(name, value) {
    try {
      Services.prefs.setIntPref(PREF_ROOT + name, Math.max(0, Math.min(100, value)));
    } catch (error) {
      console.warn("[AurexalisCore] setInt", name, error);
    }
  }

  /** Lee una cadena de preferencias. */
  function getString(name, fallback) {
    try {
      return Services.prefs.getStringPref(PREF_ROOT + name, fallback);
    } catch (_error) {
      return fallback;
    }
  }

  /** Guarda una cadena en preferencias. */
  function setString(name, value) {
    try {
      Services.prefs.setStringPref(PREF_ROOT + name, value);
    } catch (error) {
      console.warn("[AurexalisCore] setString", name, error);
    }
    if (name.startsWith("blocker.") && window.AurexalisBlocker) {
      AurexalisBlocker.applyBlockerPrefs();
    }
  }

  /** Snapshot de ajustes de sonido y UI. */
  function readSoundSettings() {
    return {
      enabled: getBool("sounds.enabled", true),
      master: getInt("sounds.master", 22) / 100,
      click: {
        enabled: getBool("sounds.click.enabled", true),
        volume: getInt("sounds.click.volume", 85) / 100,
      },
      hover: {
        enabled: getBool("sounds.hover.enabled", true),
        volume: getInt("sounds.hover.volume", 55) / 100,
      },
      key: {
        enabled: getBool("sounds.key.enabled", true),
        volume: getInt("sounds.key.volume", 40) / 100,
      },
      ambient: {
        enabled: getBool("sounds.ambient.enabled", true),
        volume: getInt("sounds.ambient.volume", 12) / 100,
      },
      panel: {
        enabled: getBool("sounds.panel.enabled", true),
        volume: getInt("sounds.panel.volume", 70) / 100,
      },
      animations: getBool("ui.animations", true),
    };
  }

  /** Ejecuta el launcher Aurexalis con argumentos (importacion, etc.). */
  function runShell(args) {
    const launcher = getString("shell.path", "");
    if (!launcher) {
      throw new Error("No se encontro aurexalis.exe en la instalacion");
    }

    const file = Cc["@mozilla.org/file/local;1"].createInstance(Ci.nsIFile);
    file.initWithPath(launcher);
    if (!file.exists()) {
      throw new Error(`Launcher inexistente: ${launcher}`);
    }

    const process = Cc["@mozilla.org/process/util;1"].createInstance(Ci.nsIProcess);
    process.init(file);
    process.runw(false, args, args.length);
    return true;
  }

  /** Abre la pagina de ajustes Aurexalis en una pestana. */
  function openSettingsPage() {
    const url = getString("settings.url", "");
    if (!url) {
      throw new Error("Pagina de ajustes no configurada");
    }

    if (window.openTrustedLinkIn) {
      window.openTrustedLinkIn(url, "tab", {
        relatedToCurrent: true,
      });
      return;
    }

    if (window.gBrowser) {
      window.gBrowser.selectedTab = window.gBrowser.addTab(url, {
        triggeringPrincipal: Services.scriptSecurityManager.getSystemPrincipal(),
      });
    }
  }

  window.AurexalisCore = {
    getBool,
    setBool,
    getInt,
    setInt,
    getString,
    setString,
    readSoundSettings,
    runShell,
    openSettingsPage,
    PREF_ROOT,
  };
})();
