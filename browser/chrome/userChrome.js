// Carga ordenada los modulos Aurexalis en el chrome del perfil.

(function initAurexalisLoader() {
  "use strict";

  if (window.__aurexalisLoaderDone) {
    return;
  }
  window.__aurexalisLoaderDone = true;

  const scripts = [
    "aurexalis-00-core.uc.js",
    "aurexalis-01-brand.uc.js",
    "aurexalis-02-blocker.uc.js",
    "aurexalis-03-sound.uc.js",
    "aurexalis-04-settings-panel.uc.js",
    "aurexalis-05-sidebar.uc.js",
    "aurexalis-06-settings-inject.uc.js",
    "aurexalis-07-cws-brand.uc.js",
  ];

  try {
    const chromeDir = Services.dirsvc.get("UChrm", Ci.nsIFile);
    const loader = Cc["@mozilla.org/moz/jssubscript-loader;1"].getService(
      Ci.mozIJSSubScriptLoader
    );

    for (const name of scripts) {
      const file = chromeDir.clone();
      file.append(name);
      if (!file.exists()) {
        console.warn("[AurexalisLoader] Missing", name);
        continue;
      }
      loader.loadSubScript(Services.io.newFileURI(file).spec);
    }
    console.info("[AurexalisLoader] Modulos cargados");
  } catch (error) {
    console.error("[AurexalisLoader]", error);
  }
})();
