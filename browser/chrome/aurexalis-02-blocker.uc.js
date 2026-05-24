// Aplica bloqueo de anuncios y rastreadores segun prefs Aurexalis (Gecko ETP + adblock-rust listo).

(function initAurexalisBlocker() {
  "use strict";

  if (window.__aurexalisBlockerLoaded) {
    return;
  }
  window.__aurexalisBlockerLoaded = true;

  const PREF_ENABLED = "aurexalis.blocker.enabled";
  const PREF_LEVEL = "aurexalis.blocker.level";
  const PREF_COSMETIC = "aurexalis.blocker.cosmetic";

  /** Niveles: off | standard | strict */
  function applyBlockerPrefs() {
    let enabled = true;
    let level = "standard";
    let cosmetic = true;

    try {
      enabled = Services.prefs.getBoolPref(PREF_ENABLED, true);
      level = Services.prefs.getStringPref(PREF_LEVEL, "standard");
      cosmetic = Services.prefs.getBoolPref(PREF_COSMETIC, true);
    } catch (_error) {
      // defaults arriba
    }

    if (!enabled || level === "off") {
      try {
        Services.prefs.setBoolPref("privacy.trackingprotection.enabled", false);
        Services.prefs.setStringPref("browser.contentblocking.category", "standard");
      } catch (error) {
        console.warn("[AurexalisBlocker] disable", error);
      }
      return;
    }

    const category = level === "strict" ? "strict" : "standard";
    try {
      Services.prefs.setBoolPref("privacy.trackingprotection.enabled", true);
      Services.prefs.setBoolPref("privacy.trackingprotection.socialtracking.enabled", true);
      Services.prefs.setStringPref("browser.contentblocking.category", category);
      Services.prefs.setBoolPref("browser.contentblocking.features.strict", category === "strict");
      Services.prefs.setBoolPref(
        "browser.contentblocking.category.standard.tracking",
        category !== "strict"
      );
      if (cosmetic) {
        Services.prefs.setBoolPref("layout.css.has-selector", true);
      }
    } catch (error) {
      console.warn("[AurexalisBlocker] apply", error);
    }

    console.info(`[AurexalisBlocker] ${category} (cosmetic=${cosmetic})`);
  }

  applyBlockerPrefs();

  try {
    Services.prefs.addObserver("aurexalis.blocker.", () => applyBlockerPrefs());
  } catch (_error) {
    // ignore
  }

  window.AurexalisBlocker = { applyBlockerPrefs };
})();
