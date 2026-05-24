// Rebrand de prompts Chrome Web Store (capa Floorp) con identidad Aurexalis.
// Requiere motor Floorp con topic floorp-chrome-web-store-install-started.

(function initAurexalisCwsBrand() {
  "use strict";

  if (window.__aurexalisCwsBrandLoaded) {
    return;
  }
  window.__aurexalisCwsBrandLoaded = true;

  const PREF_ENABLED = "aurexalis.cws.brandPrompts";
  const TOPIC_INSTALL = "floorp-chrome-web-store-install-started";
  const TOPIC_PERMISSION = "webextension-permission-prompt";

  const TEXTS = {
    defaultName: "esta extension de Chrome",
    badge: "Extension Chrome",
    message: (name) =>
      `Instalar ${name} desde Chrome Web Store en Aurexalis.`,
    prompt: (name) =>
      `${name} solicita permisos. Es una extension de Chrome instalada en Aurexalis; puede no ser totalmente compatible con Gecko.`,
    warning:
      "Las extensiones de Chrome no garantizan compatibilidad total con Aurexalis. Algunas APIs de Chromium pueden fallar.",
  };

  /** Contexto global compartido entre ventanas (patron Floorp addons/observer). */
  function getGlobalScope() {
    return Cu.getGlobalForObject(Services);
  }

  function getContext() {
    const global = getGlobalScope();
    if (!global.__aurexalisCwsContext) {
      global.__aurexalisCwsContext = {
        pending: null,
        observer: null,
        savedDescriptionChildren: null,
      };
    }
    return global.__aurexalisCwsContext;
  }

  function isEnabled() {
    try {
      return Services.prefs.getBoolPref(PREF_ENABLED, true);
    } catch (_error) {
      return true;
    }
  }

  function getPendingInfo() {
    return getContext().pending || null;
  }

  function clearPendingInfo() {
    getContext().pending = null;
  }

  function saveDescriptionChildren(container) {
    const ctx = getContext();
    if (ctx.savedDescriptionChildren !== null || !container) {
      return;
    }
    ctx.savedDescriptionChildren = Array.from(container.childNodes).map((node) =>
      node.cloneNode(true)
    );
  }

  function addBadge(notification) {
    if (notification.querySelector(".ax-cws-badge")) {
      return;
    }
    const addonList = document.getElementById("addon-install-confirmation-content");
    if (!addonList) {
      return;
    }
    const names = addonList.querySelectorAll(".addon-install-confirmation-name");
    for (const nameElement of names) {
      const badge = document.createXULElement("label");
      badge.setAttribute("value", TEXTS.badge);
      badge.setAttribute("class", "ax-cws-badge chrome-extension-badge");
      nameElement.parentElement?.appendChild(badge);
    }
  }

  function addWarning(notification, body) {
    if (notification.querySelector(".ax-cws-warning")) {
      return;
    }
    const warning = document.createXULElement("description");
    warning.setAttribute("class", "ax-cws-warning chrome-extension-warning");
    warning.textContent = TEXTS.warning;
    body.appendChild(warning);
  }

  function setMessage(container, text) {
    if (!container) {
      return;
    }
    let message = container.querySelector(".ax-cws-message");
    if (!message) {
      message = document.createElement("span");
      message.className = "ax-cws-message chrome-web-store-message";
      container.textContent = "";
      container.appendChild(message);
    }
    message.textContent = text;
  }

  /** Personaliza dialog de confirmacion de instalacion. */
  function customizeInstallConfirmation(info) {
    const notification = document.getElementById("addon-install-confirmation-notification");
    if (!notification) {
      return;
    }
    const body = notification.querySelector("popupnotificationcontent");
    if (!body) {
      return;
    }
    const name = info?.name || TEXTS.defaultName;
    setMessage(
      notification.querySelector(".popup-notification-description"),
      TEXTS.message(name)
    );
    addBadge(notification);
    addWarning(notification, body);
  }

  /** Personaliza dialog de permisos WebExtension. */
  function customizePermissionPrompt(info) {
    const notification = document.getElementById("addon-webext-permissions-notification");
    if (!notification) {
      return;
    }
    const body = notification.querySelector("popupnotificationcontent");
    if (!body) {
      return;
    }
    const name = info?.name || TEXTS.defaultName;
    const description = notification.querySelector(".popup-notification-description");
    saveDescriptionChildren(description);
    setMessage(description, TEXTS.prompt(name));

    const intro = notification.querySelector("#addon-webext-perm-intro");
    if (intro) {
      intro.style.setProperty("display", "none");
    }

    addWarning(notification, body);
    notification.setAttribute("data-ax-cws-customized", "true");
  }

  function cleanupCustomization() {
    const ctx = getContext();
    const notification = document.getElementById("addon-webext-permissions-notification");
    if (!notification) {
      return;
    }
    const description = notification.querySelector(".popup-notification-description");
    const custom = description?.querySelector(".ax-cws-message");
    if (custom && description && ctx.savedDescriptionChildren) {
      description.textContent = "";
      for (const child of ctx.savedDescriptionChildren) {
        description.appendChild(child.cloneNode(true));
      }
      ctx.savedDescriptionChildren = null;
    }
    notification.querySelector(".ax-cws-warning")?.remove();
    notification.removeAttribute("data-ax-cws-customized");
    const intro = notification.querySelector("#addon-webext-perm-intro");
    if (intro) {
      intro.style.removeProperty("display");
    }
  }

  function createObserver() {
    const ctx = getContext();
    if (ctx.observer) {
      return ctx.observer;
    }

    const observer = {
      observe(subject, topic) {
        if (!isEnabled()) {
          return;
        }

        if (topic === TOPIC_INSTALL) {
          const wrapped = subject;
          if (wrapped?.wrappedJSObject) {
            ctx.pending = wrapped.wrappedJSObject;
          }
          return;
        }

        if (topic !== TOPIC_PERMISSION) {
          return;
        }

        const pending = getPendingInfo();
        requestAnimationFrame(() => {
          requestAnimationFrame(() => {
            if (pending) {
              saveDescriptionChildren(
                document
                  .getElementById("addon-webext-permissions-notification")
                  ?.querySelector(".popup-notification-description")
              );
              customizePermissionPrompt(pending);
              clearPendingInfo();
            } else {
              cleanupCustomization();
              clearPendingInfo();
            }
          });
        });
      },
    };

    ctx.observer = observer;
    Services.obs.addObserver(observer, TOPIC_INSTALL);
    Services.obs.addObserver(observer, TOPIC_PERMISSION);
    return observer;
  }

  /** Envuelve gXPInstallObserver para rebranding en confirmacion XPInstall. */
  function overrideInstallConfirmation() {
    const gXPInstallObserver = window.gXPInstallObserver;
    if (!gXPInstallObserver || gXPInstallObserver.__aurexalisCwsWrapped) {
      return;
    }

    const original = gXPInstallObserver.showInstallConfirmation.bind(gXPInstallObserver);
    gXPInstallObserver.showInstallConfirmation = (browser, installInfo, height) => {
      const pending = getPendingInfo();
      original(browser, installInfo, height);
      if (!pending || !isEnabled()) {
        return;
      }
      requestAnimationFrame(() => {
        customizeInstallConfirmation(pending);
        clearPendingInfo();
      });
    };
    gXPInstallObserver.__aurexalisCwsWrapped = true;
  }

  function init() {
    if (!isEnabled()) {
      console.info("[AurexalisCWS] brandPrompts desactivado");
      return;
    }
    createObserver();
    overrideInstallConfirmation();
    console.info("[AurexalisCWS] Capa de rebranding CWS activa");
  }

  if (document.readyState === "complete") {
    init();
  } else {
    globalThis.addEventListener("load", () => init(), { once: true });
  }
})();
