// Panel de ajustes Aurexalis en el sidebar (sonidos, animaciones, importacion).

(function initAurexalisSettingsPanel() {
  "use strict";

  if (!window.AurexalisCore) {
    return;
  }

  function xul(name) {
    return document.createXULElement
      ? document.createXULElement(name)
      : document.createElement(name);
  }

  function labeledCheckbox(label, pref, fallback) {
    const row = xul("hbox");
    row.className = "ax-settings-row";
    row.setAttribute("align", "center");

    const box = xul("checkbox");
    box.setAttribute("label", label);
    box.checked = AurexalisCore.getBool(pref, fallback);
    box.addEventListener("command", () => {
      AurexalisCore.setBool(pref, box.checked);
      if (window.AurexalisSound) {
        AurexalisSound.refreshAmbient();
        if (pref === "ui.animations" || pref.endsWith("animations")) {
          document.documentElement.classList.toggle(
            "ax-animations-off",
            !AurexalisCore.getBool("ui.animations", true)
          );
        }
      }
      if (window.AurexalisSound && pref.includes("panel")) {
        AurexalisSound.play("panel");
      }
    });

    row.appendChild(box);
    return row;
  }

  function volumeRow(label, pref, fallback) {
    const row = xul("hbox");
    row.className = "ax-settings-row";
    row.setAttribute("align", "center");

    const caption = xul("label");
    caption.setAttribute("value", label);
    caption.setAttribute("flex", "1");

    const scale = xul("scale");
    scale.setAttribute("min", "0");
    scale.setAttribute("max", "100");
    scale.setAttribute("value", String(AurexalisCore.getInt(pref, fallback)));
    scale.addEventListener("input", () => {
      AurexalisCore.setInt(pref, Number(scale.value));
    });

    row.appendChild(caption);
    row.appendChild(scale);
    return row;
  }

  function levelSelect() {
    const row = xul("hbox");
    row.className = "ax-settings-row";
    row.setAttribute("align", "center");

    const caption = xul("label");
    caption.setAttribute("value", "Nivel bloqueo");
    caption.setAttribute("flex", "1");

    const menulist = xul("menulist");
    const menupopup = xul("menupopup");
    for (const [val, text] of [
      ["standard", "Estándar"],
      ["strict", "Estricto"],
      ["off", "Apagado"],
    ]) {
      const item = xul("menuitem");
      item.setAttribute("label", text);
      item.setAttribute("value", val);
      menupopup.appendChild(item);
    }
    menulist.appendChild(menupopup);
    menulist.value = AurexalisCore.getString("blocker.level", "standard");
    menulist.addEventListener("command", () => {
      AurexalisCore.setString("blocker.level", menulist.value);
    });

    row.appendChild(caption);
    row.appendChild(menulist);
    return row;
  }

  function actionButton(label, handler) {
    const button = xul("button");
    button.className = "ax-panel-action";
    button.setAttribute("label", label);
    button.addEventListener("command", handler);
    return button;
  }

  function runImport(includePasswords) {
    try {
      const args = ["import", "audit"];
      if (includePasswords) {
        args.push("--passwords");
      }
      AurexalisCore.runShell(args);
      if (window.AurexalisSound) {
        AurexalisSound.play("panel");
      }
    } catch (error) {
      console.error("[AurexalisSettings]", error);
      Services.prompt.alert(null, "Aurexalis", String(error));
    }
  }

  window.AurexalisSettingsPanel = {
    title: "Ajustes Aurexalis",
    build(container) {
      while (container.firstChild) {
        container.firstChild.remove();
      }

      const intro = xul("description");
      intro.textContent =
        "Identidad Aurexalis: morado, rojo y dorado. Ajustes locales sin telemetria.";
      container.appendChild(intro);

      container.appendChild(labeledCheckbox("Sonidos activos", "sounds.enabled", true));
      container.appendChild(volumeRow("Volumen general", "sounds.master", 22));
      container.appendChild(labeledCheckbox("Clicks", "sounds.click.enabled", true));
      container.appendChild(volumeRow("Vol. clicks", "sounds.click.volume", 85));
      container.appendChild(labeledCheckbox("Hover", "sounds.hover.enabled", true));
      container.appendChild(volumeRow("Vol. hover", "sounds.hover.volume", 55));
      container.appendChild(labeledCheckbox("Teclado", "sounds.key.enabled", true));
      container.appendChild(volumeRow("Vol. teclado", "sounds.key.volume", 40));
      container.appendChild(labeledCheckbox("Ambiente", "sounds.ambient.enabled", true));
      container.appendChild(volumeRow("Vol. ambiente", "sounds.ambient.volume", 12));
      container.appendChild(labeledCheckbox("Panel lateral", "sounds.panel.enabled", true));
      container.appendChild(volumeRow("Vol. panel", "sounds.panel.volume", 70));
      container.appendChild(labeledCheckbox("Animaciones UI", "ui.animations", true));

      const blockerSep = xul("separator");
      blockerSep.className = "ax-settings-sep";
      container.appendChild(blockerSep);

      const blockerTitle = xul("label");
      blockerTitle.className = "ax-panel-row-key";
      blockerTitle.setAttribute("value", "Bloqueador");
      container.appendChild(blockerTitle);

      container.appendChild(labeledCheckbox("Bloqueador activo", "blocker.enabled", true));
      container.appendChild(levelSelect());
      container.appendChild(
        labeledCheckbox("Filtros cosmeticos", "blocker.cosmetic", true)
      );

      const sep = xul("separator");
      sep.className = "ax-settings-sep";
      container.appendChild(sep);

      const importTitle = xul("label");
      importTitle.className = "ax-panel-row-key";
      importTitle.setAttribute("value", "Importacion local");
      container.appendChild(importTitle);

      container.appendChild(
        actionButton("Exportar datos Chromium (sin contrasenas)", () => runImport(false))
      );
      container.appendChild(
        actionButton("Exportar incl. contrasenas (consentimiento)", () => {
          const ok = Services.prompt.confirm(
            null,
            "Aurexalis — contrasenas",
            "Se exportaran contrasenas Chromium a un JSON local auditable en tu perfil. " +
              "No se envia nada a Internet. ¿Continuar?"
          );
          if (ok) {
            runImport(true);
          }
        })
      );

      container.appendChild(
        actionButton("Abrir pagina de ajustes", () => {
          try {
            AurexalisCore.openSettingsPage();
          } catch (error) {
            Services.prompt.alert(null, "Aurexalis", String(error));
          }
        })
      );
    },
  };
})();
