// Desactiva Floorp Start para usar la pagina local de Aurexalis (browser/newtab.url).

(function initAurexalisHome() {
  "use strict";

  if (window.__aurexalisHomeLoaded) {
    return;
  }
  window.__aurexalisHomeLoaded = true;

  const DESIGN_PREF = "floorp.design.configs";
  let observerBound = false;

  /** Marca disableFloorpStart en la config JSON de Floorp. */
  function patchFloorpStart() {
    try {
      if (!Services.prefs.prefHasUserValue(DESIGN_PREF)) {
        const branch = Services.prefs.getDefaultBranch("");
        if (!branch.prefHasUserValue(DESIGN_PREF)) {
          return false;
        }
      }

      const raw = Services.prefs.getStringPref(DESIGN_PREF, "");
      if (!raw) {
        return false;
      }

      const data = JSON.parse(raw);
      if (!data.uiCustomization) {
        data.uiCustomization = {};
      }
      if (data.uiCustomization.disableFloorpStart === true) {
        return true;
      }

      data.uiCustomization.disableFloorpStart = true;
      Services.prefs.setStringPref(DESIGN_PREF, JSON.stringify(data));
      return true;
    } catch (error) {
      console.error("[Aurexalis] No se pudo desactivar Floorp Start:", error);
      return false;
    }
  }

  /** Reintenta hasta que Floorp haya escrito su config inicial. */
  function ensureFloorpStartDisabled() {
    if (patchFloorpStart()) {
      return;
    }

    let attempts = 0;
    const timer = setInterval(() => {
      attempts += 1;
      if (patchFloorpStart() || attempts >= 40) {
        clearInterval(timer);
      }
    }, 500);
  }

  if (Services.appinfo.processType !== Services.appinfo.PROCESS_TYPE_DEFAULT) {
    return;
  }

  ensureFloorpStartDisabled();

  if (!observerBound) {
    Services.prefs.addObserver(DESIGN_PREF, ensureFloorpStartDisabled);
    observerBound = true;
  }
})();
