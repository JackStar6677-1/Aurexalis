// Aurexalis: sonidos de click, hover, teclas, panel y ambiente (OGG o sintesis WebAudio).

(function initAurexalisSound() {
  "use strict";

  const fileSounds = {
    click: "sounds/click.ogg",
    hover: "sounds/hover.ogg",
    key: "sounds/key.ogg",
    panel: "sounds/panel.ogg",
    ambient: "sounds/ambient.ogg",
  };

  const state = {
    context: null,
    buffers: new Map(),
    ambientNodes: null,
    lastHoverAt: 0,
    minHoverIntervalMs: 70,
  };

  function settings() {
    return window.AurexalisCore
      ? window.AurexalisCore.readSoundSettings()
      : {
          enabled: true,
          master: 0.22,
          click: { enabled: true, volume: 0.85 },
          hover: { enabled: true, volume: 0.55 },
          key: { enabled: true, volume: 0.4 },
          ambient: { enabled: true, volume: 0.12 },
          panel: { enabled: true, volume: 0.7 },
          animations: true,
        };
  }

  function chromeBase() {
    try {
      return Services.io.newURI(document.location.href).resolve("./");
    } catch (_error) {
      return "";
    }
  }

  function ensureContext() {
    try {
      if (!state.context) {
        state.context = new AudioContext();
      }
      if (state.context.state === "suspended") {
        state.context.resume().catch(() => {});
      }
      return state.context;
    } catch (_error) {
      return null;
    }
  }

  /** Sintetiza un sonido corto si no hay archivo OGG. */
  function synthTone(kind) {
    const ctx = ensureContext();
    if (!ctx) {
      return;
    }

    const now = ctx.currentTime;
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    const filter = ctx.createBiquadFilter();

    const presets = {
      click: { type: "triangle", f0: 520, f1: 240, dur: 0.06, vol: 0.35 },
      hover: { type: "sine", f0: 880, f1: 1180, dur: 0.04, vol: 0.18 },
      key: { type: "square", f0: 640, f1: 720, dur: 0.03, vol: 0.12 },
      panel: { type: "triangle", f0: 300, f1: 420, dur: 0.12, vol: 0.28 },
    };
    const preset = presets[kind] || presets.click;

    osc.type = preset.type;
    osc.frequency.setValueAtTime(preset.f0, now);
    osc.frequency.exponentialRampToValueAtTime(preset.f1, now + preset.dur);
    filter.type = "lowpass";
    filter.frequency.value = 2400;
    gain.gain.setValueAtTime(preset.vol, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + preset.dur + 0.02);

    osc.connect(filter);
    filter.connect(gain);
    gain.connect(ctx.destination);
    osc.start(now);
    osc.stop(now + preset.dur + 0.03);
  }

  /** Bucle ambiental suave (pad sintetico). */
  function startAmbient() {
    stopAmbient();
    const cfg = settings();
    if (!cfg.enabled || !cfg.ambient.enabled) {
      return;
    }

    const ctx = ensureContext();
    if (!ctx) {
      return;
    }

    const master = cfg.master * cfg.ambient.volume;
    const oscA = ctx.createOscillator();
    const oscB = ctx.createOscillator();
    const gain = ctx.createGain();
    const filter = ctx.createBiquadFilter();

    oscA.type = "sine";
    oscB.type = "sine";
    oscA.frequency.value = 110;
    oscB.frequency.value = 164.81;
    filter.type = "lowpass";
    filter.frequency.value = 420;
    gain.gain.value = master * 0.08;

    oscA.connect(filter);
    oscB.connect(filter);
    filter.connect(gain);
    gain.connect(ctx.destination);
    oscA.start();
    oscB.start();

    state.ambientNodes = { oscA, oscB, gain, filter };
  }

  function stopAmbient() {
    if (!state.ambientNodes) {
      return;
    }
    try {
      state.ambientNodes.oscA.stop();
      state.ambientNodes.oscB.stop();
    } catch (_error) {
      // ignore
    }
    state.ambientNodes = null;
  }

  async function loadBuffer(name, relativePath) {
    const ctx = ensureContext();
    if (!ctx) {
      return;
    }
    try {
      const response = await fetch(chromeBase() + relativePath);
      if (!response.ok) {
        return;
      }
      const data = await response.arrayBuffer();
      const buffer = await ctx.decodeAudioData(data);
      state.buffers.set(name, buffer);
    } catch (_error) {
      // fallback a sintesis
    }
  }

  function play(kind) {
    const cfg = settings();
    if (!cfg.enabled) {
      return;
    }

    const channel = cfg[kind];
    if (!channel || !channel.enabled) {
      return;
    }

    const volume = cfg.master * channel.volume;
    const buffer = state.buffers.get(kind);
    const ctx = ensureContext();
    if (!ctx) {
      return;
    }

    if (buffer) {
      const source = ctx.createBufferSource();
      const gain = ctx.createGain();
      gain.gain.value = volume;
      source.buffer = buffer;
      source.connect(gain);
      gain.connect(ctx.destination);
      source.start(0);
      return;
    }

    synthTone(kind);
  }

  function isInteractive(target) {
    try {
      return Boolean(
        target.closest(
          "toolbarbutton, button, .tabbrowser-tab, #urlbar, menuitem, richlistitem, .ax-sidebar-button, .ax-panel-action, .ax-settings-row"
        )
      );
    } catch (_error) {
      return false;
    }
  }

  function refreshAmbient() {
    const cfg = settings();
    if (cfg.enabled && cfg.ambient.enabled) {
      startAmbient();
    } else {
      stopAmbient();
    }
  }

  async function preload() {
    await Promise.all(
      Object.entries(fileSounds).map(([name, path]) => loadBuffer(name, path))
    );
    refreshAmbient();
  }

  function observePrefs() {
    if (!window.AurexalisCore) {
      return;
    }
    try {
      Services.prefs.addObserver(AurexalisCore.PREF_ROOT, () => {
        refreshAmbient();
        document.documentElement.classList.toggle(
          "ax-animations-off",
          !settings().animations
        );
      });
    } catch (_error) {
      // ignore
    }
  }

  window.addEventListener(
    "click",
    (event) => {
      if (isInteractive(event.target)) {
        play("click");
      }
    },
    true
  );

  window.addEventListener(
    "mouseover",
    (event) => {
      const now = performance.now();
      if (now - state.lastHoverAt < state.minHoverIntervalMs) {
        return;
      }
      if (isInteractive(event.target)) {
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
    document.documentElement.classList.toggle(
      "ax-animations-off",
      !settings().animations
    );
    preload().catch(() => {});
    observePrefs();
  });

  window.AurexalisSound = { play, refreshAmbient };
})();
