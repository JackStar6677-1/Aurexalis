// Inyecta puente de prefs en la pagina file:// de ajustes Aurexalis.

(function initAurexalisSettingsInject() {
  "use strict";

  if (window.__aurexalisSettingsInjectLoaded) {
    return;
  }
  window.__aurexalisSettingsInjectLoaded = true;

  const PREF_ROOT = "aurexalis.";

  function bridgeApi() {
    return {
      getBool(name, fallback) {
        try {
          return Services.prefs.getBoolPref(PREF_ROOT + name, fallback);
        } catch (_error) {
          return fallback;
        }
      },
      setBool(name, value) {
        try {
          Services.prefs.setBoolPref(PREF_ROOT + name, value);
        } catch (error) {
          console.warn("[AurexalisSettingsInject]", error);
        }
        if (name.startsWith("blocker.") && window.AurexalisBlocker) {
          AurexalisBlocker.applyBlockerPrefs();
        }
        if (name.startsWith("sounds.") && window.AurexalisSound) {
          AurexalisSound.refreshAmbient();
        }
        if (name === "ui.animations") {
          document.documentElement.classList.toggle(
            "ax-animations-off",
            !Services.prefs.getBoolPref(PREF_ROOT + "ui.animations", true)
          );
        }
      },
      getInt(name, fallback) {
        try {
          return Services.prefs.getIntPref(PREF_ROOT + name, fallback);
        } catch (_error) {
          return fallback;
        }
      },
      setInt(name, value) {
        try {
          Services.prefs.setIntPref(PREF_ROOT + name, Math.max(0, Math.min(100, value)));
        } catch (error) {
          console.warn("[AurexalisSettingsInject]", error);
        }
      },
      getString(name, fallback) {
        try {
          return Services.prefs.getStringPref(PREF_ROOT + name, fallback);
        } catch (_error) {
          return fallback;
        }
      },
      setString(name, value) {
        try {
          Services.prefs.setStringPref(PREF_ROOT + name, value);
        } catch (error) {
          console.warn("[AurexalisSettingsInject]", error);
        }
        if (name === "blocker.level" && window.AurexalisBlocker) {
          AurexalisBlocker.applyBlockerPrefs();
        }
      },
    };
  }

  function injectIntoBrowser(browser) {
    if (!browser?.contentDocument) {
      return;
    }
    const url = browser.currentURI?.spec || "";
    if (!url.includes("/settings/index.html")) {
      return;
    }
    const win = browser.contentWindow;
    if (!win || win.AurexalisPrefsBridge) {
      return;
    }
    win.AurexalisPrefsBridge = bridgeApi();
    win.dispatchEvent(new CustomEvent("aurexalis-prefs-ready"));
  }

  function registerTabListener() {
    const gBrowser = window.gBrowser;
    if (!gBrowser?.tabContainer) {
      return;
    }
    gBrowser.tabContainer.addEventListener("TabSelect", () => {
      try {
        injectIntoBrowser(gBrowser.selectedBrowser);
      } catch (_error) {
        // ignore
      }
    });
    gBrowser.addTabsProgressListener({
      onStateChange(browser, webProgress, stateFlags) {
        if (stateFlags & Ci.nsIWebProgressListener.STATE_STOP) {
          injectIntoBrowser(browser);
        }
      },
    });
  }

  if (document.readyState === "complete") {
    registerTabListener();
  } else {
    window.addEventListener("load", registerTabListener, { once: true });
  }
})();
