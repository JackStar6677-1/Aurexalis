// Aurexalis reactive sound scaffold for Firefox/Floorp chrome UI experiments.
// Expected local files: chrome/sounds/click.ogg, hover.ogg, key.ogg

(function initAurexalisSound() {
  "use strict";

  const config = {
    enabled: true,
    volume: 0.18,
    minHoverIntervalMs: 80,
    sounds: {
      click: "sounds/click.ogg",
      hover: "sounds/hover.ogg",
      key: "sounds/key.ogg",
    },
  };

  const state = {
    context: null,
    buffers: new Map(),
    lastHoverAt: 0,
  };

  function getChromeBaseUrl() {
    try {
      const uri = Services.io.newURI(document.location.href);
      return uri.resolve("./");
    } catch (error) {
      console.warn("[AurexalisSound] Cannot resolve chrome base URL", error);
      return "";
    }
  }

  async function loadBuffer(name, relativePath) {
    try {
      const response = await fetch(getChromeBaseUrl() + relativePath);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }
      const data = await response.arrayBuffer();
      const buffer = await state.context.decodeAudioData(data);
      state.buffers.set(name, buffer);
      console.info(`[AurexalisSound] Loaded ${name}`);
    } catch (error) {
      console.warn(`[AurexalisSound] Skipped ${name}: ${error.message}`);
    }
  }

  function ensureContext() {
    try {
      if (!state.context) {
        state.context = new AudioContext();
      }
      if (state.context.state === "suspended") {
        state.context.resume().catch((error) => {
          console.warn("[AurexalisSound] Cannot resume AudioContext", error);
        });
      }
      return true;
    } catch (error) {
      console.warn("[AurexalisSound] AudioContext unavailable", error);
      return false;
    }
  }

  function play(name) {
    try {
      if (!config.enabled || !ensureContext()) {
        return;
      }
      const buffer = state.buffers.get(name);
      if (!buffer) {
        return;
      }

      const source = state.context.createBufferSource();
      const gain = state.context.createGain();
      gain.gain.value = config.volume;
      source.buffer = buffer;
      source.connect(gain);
      gain.connect(state.context.destination);
      source.start(0);
    } catch (error) {
      console.warn(`[AurexalisSound] Cannot play ${name}`, error);
    }
  }

  function isInteractiveElement(target) {
    try {
      return Boolean(
        target.closest("toolbarbutton, button, .tabbrowser-tab, #urlbar, menuitem, richlistitem")
      );
    } catch (_error) {
      return false;
    }
  }

  async function preload() {
    if (!ensureContext()) {
      return;
    }
    await Promise.all(
      Object.entries(config.sounds).map(([name, path]) => loadBuffer(name, path))
    );
  }

  window.addEventListener(
    "click",
    (event) => {
      if (isInteractiveElement(event.target)) {
        play("click");
      }
    },
    true
  );

  window.addEventListener(
    "mouseover",
    (event) => {
      const now = performance.now();
      if (now - state.lastHoverAt < config.minHoverIntervalMs) {
        return;
      }
      if (isInteractiveElement(event.target)) {
        state.lastHoverAt = now;
        play("hover");
      }
    },
    true
  );

  window.addEventListener(
    "keydown",
    (event) => {
      if (!event.repeat && event.key.length === 1) {
        play("key");
      }
    },
    true
  );

  window.addEventListener("load", () => {
    preload().catch((error) => {
      console.warn("[AurexalisSound] Preload failed", error);
    });
  });
})();

