// Bloqueador Aurexalis: ETP (Gecko) + hook de red con listas sync-lists (subset ABP).
// Paridad completa adblock-rust: CLI `aurexalis blocker check` y futuro sidecar/XPI.

(function initAurexalisBlocker() {
  "use strict";

  if (window.__aurexalisBlockerLoaded) {
    return;
  }
  window.__aurexalisBlockerLoaded = true;

  const PREF_ENABLED = "aurexalis.blocker.enabled";
  const PREF_LEVEL = "aurexalis.blocker.level";
  const PREF_COSMETIC = "aurexalis.blocker.cosmetic";
  const FILTER_REL = "blocker/aurexalis-filters.txt";
  const TOPIC_HTTP_MODIFY = "http-on-modify-request";

  let networkRules = [];
  let httpObserverRegistered = false;

  const httpObserver = {
    observe(subject, topic) {
      if (topic !== TOPIC_HTTP_MODIFY) {
        return;
      }
      try {
        const channel = subject.QueryInterface(Ci.nsIHttpChannel);
        const requestUrl = channel.URI.spec;
        const sourceUrl = channel.referrer?.spec ?? null;
        const kind = guessResourceKind(channel);
        const decision = matchNetworkRules(requestUrl, sourceUrl, kind);
        if (decision.block) {
          channel.cancel(Cr.NS_BINDING_ABORTED);
          if (Services.prefs.getBoolPref("aurexalis.blocker.debug", false)) {
            console.info(
              `[AurexalisBlocker] blocked ${requestUrl} (${decision.rule})`
            );
          }
        }
      } catch (error) {
        if (Services.prefs.getBoolPref("aurexalis.blocker.debug", false)) {
          console.warn("[AurexalisBlocker] observe", error);
        }
      }
    },
  };

  /** Alineado con `matches_rule` del crate aurexalis-blocker (backend Simple). */
  function splitOptions(rule) {
    const idx = rule.indexOf("$");
    if (idx === -1) {
      return { pattern: rule, options: null };
    }
    return { pattern: rule.slice(0, idx), options: rule.slice(idx + 1) };
  }

  function hostFromUrl(url) {
    try {
      return new URL(url).hostname.toLowerCase();
    } catch (_error) {
      return "";
    }
  }

  function isThirdParty(requestUrl, sourceUrl) {
    const reqHost = hostFromUrl(requestUrl);
    const srcHost = sourceUrl ? hostFromUrl(sourceUrl) : "";
    if (!reqHost || !srcHost) {
      return false;
    }
    return reqHost !== srcHost && !reqHost.endsWith(`.${srcHost}`);
  }

  function optionsMatch(options, requestUrl, sourceUrl, kind) {
    if (!options) {
      return true;
    }
    for (const raw of options.split(",")) {
      const option = raw.trim();
      if (option === "script" && kind !== "script") {
        return false;
      }
      if (option === "image" && kind !== "image") {
        return false;
      }
      if (option === "stylesheet" && kind !== "stylesheet") {
        return false;
      }
      if (option === "third-party" && !isThirdParty(requestUrl, sourceUrl)) {
        return false;
      }
    }
    return true;
  }

  function matchesRule(rule, requestUrl, sourceUrl, kind) {
    const { pattern, options } = splitOptions(rule);
    if (!optionsMatch(options, requestUrl, sourceUrl, kind)) {
      return false;
    }

    if (pattern.startsWith("||")) {
      const needle = pattern
        .slice(2)
        .replace(/\^$/, "")
        .replace(/\/$/, "");
      const host = hostFromUrl(requestUrl);
      return host === needle || host.endsWith(`.${needle}`);
    }

    return requestUrl.includes(pattern);
  }

  function matchNetworkRules(requestUrl, sourceUrl, kind) {
    for (const rule of networkRules) {
      if (rule.startsWith("@@") && matchesRule(rule.slice(2), requestUrl, sourceUrl, kind)) {
        return { block: false, rule: null };
      }
    }
    for (const rule of networkRules) {
      if (!rule.startsWith("@@") && matchesRule(rule, requestUrl, sourceUrl, kind)) {
        return { block: true, rule };
      }
    }
    return { block: false, rule: null };
  }

  function guessResourceKind(channel) {
    try {
      const type = channel.loadInfo?.externalContentPolicyType;
      if (type === Ci.nsIContentPolicy.TYPE_SCRIPT) {
        return "script";
      }
      if (type === Ci.nsIContentPolicy.TYPE_IMAGE) {
        return "image";
      }
      if (type === Ci.nsIContentPolicy.TYPE_STYLESHEET) {
        return "stylesheet";
      }
    } catch (_error) {
      // ignore
    }
    return "other";
  }

  function readTextFile(file) {
    const stream = Cc["@mozilla.org/network/file-input-stream;1"].createInstance(
      Ci.nsIFileInputStream
    );
    stream.init(file, -1, -1, 0);
    const sis = Cc["@mozilla.org/scriptableinputstream;1"].createInstance(
      Ci.nsIScriptableInputStream
    );
    sis.init(stream);
    const text = sis.read(stream.available());
    sis.close();
    stream.close();
    return text;
  }

  function filterListFile() {
    const prof = Services.dirsvc.get("ProfD", Ci.nsIFile);
    const file = prof.clone();
    for (const part of FILTER_REL.split("/")) {
      file.append(part);
    }
    return file;
  }

  /** Recarga reglas desde `ProfD/blocker/aurexalis-filters.txt` (salida de sync-lists). */
  function reloadNetworkFilters() {
    networkRules = [];
    try {
      const file = filterListFile();
      if (!file.exists()) {
        console.info("[AurexalisBlocker] sin listas en perfil; ejecuta sync-lists");
        return 0;
      }
      const body = readTextFile(file);
      networkRules = body
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter((line) => line && !line.startsWith("!"));
      console.info(`[AurexalisBlocker] ${networkRules.length} reglas de red cargadas`);
      return networkRules.length;
    } catch (error) {
      console.warn("[AurexalisBlocker] reloadNetworkFilters", error);
      return 0;
    }
  }

  function setNetworkHookActive(active) {
    try {
      if (active && !httpObserverRegistered) {
        Services.obs.addObserver(httpObserver, TOPIC_HTTP_MODIFY);
        httpObserverRegistered = true;
      } else if (!active && httpObserverRegistered) {
        Services.obs.removeObserver(httpObserver, TOPIC_HTTP_MODIFY);
        httpObserverRegistered = false;
      }
    } catch (error) {
      console.warn("[AurexalisBlocker] network hook", error);
    }
  }

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

    const networkActive = enabled && level !== "off";
    reloadNetworkFilters();
    setNetworkHookActive(networkActive && networkRules.length > 0);

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

    console.info(
      `[AurexalisBlocker] ${category} (cosmetic=${cosmetic}, network=${networkActive && networkRules.length > 0})`
    );
  }

  applyBlockerPrefs();

  try {
    Services.prefs.addObserver("aurexalis.blocker.", () => applyBlockerPrefs());
  } catch (_error) {
    // ignore
  }

  window.AurexalisBlocker = {
    applyBlockerPrefs,
    reloadNetworkFilters,
    matchNetworkRules,
    getNetworkRules: () => networkRules.slice(),
  };
})();
