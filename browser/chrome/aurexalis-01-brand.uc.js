// Oculta branding visible de Floorp y refuerza identidad Aurexalis en chrome.

(function initAurexalisBrand() {
  "use strict";

  if (window.__aurexalisBrandLoaded) {
    return;
  }
  window.__aurexalisBrandLoaded = true;

  function patchFloorpStart() {
    if (!window.AurexalisCore) {
      return;
    }
    const pref = "floorp.design.configs";
    try {
      const raw = Services.prefs.getStringPref(pref, "");
      if (!raw) {
        return;
      }
      const data = JSON.parse(raw);
      if (!data.uiCustomization) {
        data.uiCustomization = {};
      }
      data.uiCustomization.disableFloorpStart = true;
      Services.prefs.setStringPref(pref, JSON.stringify(data));
    } catch (error) {
      console.warn("[AurexalisBrand] floorp.design.configs", error);
    }
  }

  function setWindowTitle() {
    try {
      const win = Services.wm.getMostRecentWindow("navigator:browser");
      if (win && win.document) {
        win.document.title = win.document.title.replace(/Floorp/gi, "Aurexalis");
      }
    } catch (_error) {
      // ignore
    }
  }

  patchFloorpStart();
  setWindowTitle();

  Services.obs.addObserver(() => setWindowTitle(), "domwindowopened");
})();
