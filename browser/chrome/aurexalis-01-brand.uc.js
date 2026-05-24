// Reemplaza branding Floorp/Ablaze por Aurexalis en chrome, titulos y about:preferences.

(function initAurexalisBrand() {
  "use strict";

  if (window.__aurexalisBrandLoaded) {
    return;
  }
  window.__aurexalisBrandLoaded = true;

  const REPLACEMENTS = [
    [/Ablaze\s+Floorp/gi, "Aurexalis"],
    [/Ablaze\s*Floorp/gi, "Aurexalis"],
    [/\bFloorp\b/g, "Aurexalis"],
    [/\bfloorp\b/g, "aurexalis"],
    [/\bAblaze\b/g, "Aurexalis"],
  ];

  function rewriteText(value) {
    if (!value || typeof value !== "string") {
      return value;
    }
    let out = value;
    for (const [pattern, replacement] of REPLACEMENTS) {
      out = out.replace(pattern, replacement);
    }
    return out;
  }

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

  function applyIdentityPrefs() {
    const pairs = [
      ["app.feedback.baseURL", ""],
      ["browser.preferences.moreFromMozilla", false],
    ];
    for (const [name, value] of pairs) {
      try {
        if (typeof value === "boolean") {
          Services.prefs.setBoolPref(name, value);
        } else {
          Services.prefs.setStringPref(name, value);
        }
      } catch (_error) {
        // ignore locked or unknown prefs
      }
    }
  }

  function patchNode(node) {
    if (!node) {
      return;
    }
    if (node.nodeType === Node.TEXT_NODE) {
      const next = rewriteText(node.nodeValue);
      if (next !== node.nodeValue) {
        node.nodeValue = next;
      }
      return;
    }
    if (node.nodeType !== Node.ELEMENT_NODE) {
      return;
    }
    const attrs = ["title", "label", "tooltiptext", "aria-label", "alt", "value", "placeholder"];
    for (const attr of attrs) {
      if (!node.hasAttribute(attr)) {
        continue;
      }
      const current = node.getAttribute(attr);
      const next = rewriteText(current);
      if (next !== current) {
        node.setAttribute(attr, next);
      }
    }
    if (node.localName === "title" && node.textContent) {
      const next = rewriteText(node.textContent);
      if (next !== node.textContent) {
        node.textContent = next;
      }
    }
  }

  function walkRoot(root) {
    if (!root) {
      return;
    }
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT | NodeFilter.SHOW_TEXT);
    let current = walker.currentNode;
    while (current) {
      patchNode(current);
      current = walker.nextNode();
    }
  }

  function patchDocument(doc) {
    if (!doc || !doc.documentElement) {
      return;
    }
    try {
      walkRoot(doc.documentElement);
    } catch (error) {
      console.warn("[AurexalisBrand] patchDocument", error);
    }
  }

  function setWindowTitle(win) {
    try {
      if (win?.document?.title) {
        win.document.title = rewriteText(win.document.title);
      }
    } catch (_error) {
      // ignore
    }
  }

  function observeDocument(doc) {
    if (!doc || doc.__aurexalisBrandObserved) {
      return;
    }
    doc.__aurexalisBrandObserved = true;
    patchDocument(doc);
    try {
      const observer = new MutationObserver((mutations) => {
        for (const mutation of mutations) {
          if (mutation.type === "characterData") {
            patchNode(mutation.target);
          } else {
            mutation.addedNodes.forEach((node) => {
              patchNode(node);
              if (node.nodeType === Node.ELEMENT_NODE) {
                walkRoot(node);
              }
            });
          }
        }
      });
      observer.observe(doc.documentElement, {
        childList: true,
        subtree: true,
        characterData: true,
      });
    } catch (error) {
      console.warn("[AurexalisBrand] observer", error);
    }
  }

  function onWindowOpened(subject) {
    try {
      const win = subject.QueryInterface(Ci.nsIDOMWindow);
      if (!win?.document) {
        return;
      }
      setWindowTitle(win);
      observeDocument(win.document);
      win.addEventListener(
        "load",
        () => {
          setWindowTitle(win);
          observeDocument(win.document);
        },
        { once: true }
      );
    } catch (_error) {
      // ignore
    }
  }

  patchFloorpStart();
  applyIdentityPrefs();
  setWindowTitle(window);
  observeDocument(document);

  Services.obs.addObserver(onWindowOpened, "domwindowopened");

  try {
    for (const win of Services.wm.getEnumerator("navigator:browser")) {
      onWindowOpened(win);
    }
  } catch (_error) {
    // ignore
  }
})();
