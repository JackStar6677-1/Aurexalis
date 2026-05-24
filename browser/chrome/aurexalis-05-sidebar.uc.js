// Aurexalis sidebar: dock vertical con paneles nativos y ajustes integrados.

(function initAurexalisSidebar() {
  "use strict";

  if (window.__aurexalisSidebarLoaded) {
    return;
  }
  window.__aurexalisSidebarLoaded = true;

  const items = [
    { id: "home", label: "AX", title: "Inicio Aurexalis", action: openAurexalisHome },
    { id: "gx", label: "GX", title: "GX Corner", action: () => openTab("https://gxcorner.games/") },
    { id: "remote", label: "RF", title: "Archivos remotos", panel: remoteFilesPanel },
    { id: "bookmarks", label: "BM", title: "Marcadores", action: toggleBookmarks },
    { id: "downloads", label: "DL", title: "Descargas", action: openDownloads },
    { id: "importer", label: "IM", title: "Importador", panel: importerPanel },
    { id: "blocker", label: "BL", title: "Bloqueador on/off", action: toggleBlocker },
    { id: "extensions", label: "EX", title: "Extensiones y Chrome Web Store", action: openExtensions },
    { id: "passwords", label: "PW", title: "Contrasenas", action: () => openTab("about:logins") },
    { id: "settings", label: "ST", title: "Ajustes Aurexalis", panel: settingsPanel },
  ];

  function xul(name) {
    if (document.createXULElement) {
      return document.createXULElement(name);
    }
    return document.createElement(name);
  }

  function openTab(url) {
    try {
      if (window.openTrustedLinkIn) {
        window.openTrustedLinkIn(url, "tab");
        return;
      }
      if (window.gBrowser) {
        window.gBrowser.selectedTab = window.gBrowser.addTab(url, {
          triggeringPrincipal: Services.scriptSecurityManager.getSystemPrincipal(),
        });
      }
    } catch (error) {
      console.warn("[AurexalisSidebar] Cannot open tab", url, error);
    }
  }

  function openAurexalisHome() {
    try {
      const home = Services.prefs.getStringPref("browser.newtab.url", "");
      if (home) {
        openTab(home);
        return;
      }
    } catch (_error) {
      // fallback below
    }
    openTab("about:home");
  }

  function toggleBookmarks() {
    try {
      if (window.SidebarUI) {
        window.SidebarUI.toggle("viewBookmarksSidebar");
      } else {
        openTab("about:bookmarks");
      }
    } catch (error) {
      console.warn("[AurexalisSidebar] Cannot toggle bookmarks", error);
    }
  }

  function openDownloads() {
    try {
      if (window.BrowserDownloadsUI) {
        window.BrowserDownloadsUI();
      } else {
        openTab("about:downloads");
      }
    } catch (error) {
      console.warn("[AurexalisSidebar] Cannot open downloads", error);
    }
  }

  /** Abre gestor de extensiones y enlace a Chrome Web Store (motor Floorp). */
  function openExtensions() {
    openTab("about:addons");
    try {
      const store = AurexalisCore
        ? AurexalisCore.getString("cws.storeUrl", "https://chromewebstore.google.com/")
        : "https://chromewebstore.google.com/";
      if (AurexalisCore && !AurexalisCore.getBool("cws.enabled", true)) {
        return;
      }
      openTab(store);
    } catch (error) {
      console.warn("[AurexalisSidebar] Cannot open CWS", error);
    }
  }

  function runImport(includePasswords) {
    if (!window.AurexalisCore) {
      Services.prompt.alert(null, "Aurexalis", "Nucleo Aurexalis no cargado.");
      return;
    }
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
      console.error("[AurexalisSidebar] import", error);
      Services.prompt.alert(null, "Aurexalis", String(error));
    }
  }

  function remoteFilesPanel() {
    return {
      title: "Archivos remotos",
      explorer: true,
    };
  }

  /** Construye el explorador RemoteFS en el panel lateral RF. */
  function buildRemoteExplorer(panel, body, button) {
    const rows = panel.querySelector(".ax-panel-rows");
    const actions = panel.querySelector(".ax-panel-actions");
    rows.hidden = true;
    actions.hidden = false;

    let host = panel.querySelector(".ax-rf-explorer");
    if (!host) {
      host = xul("vbox");
      host.className = "ax-rf-explorer";
      body.insertBefore(host, rows);
    }
    host.hidden = false;
    while (host.firstChild) {
      host.firstChild.remove();
    }

    const settingsHost = panel.querySelector(".ax-settings-host");
    if (settingsHost) {
      settingsHost.hidden = true;
    }

    function pref(name, fallback) {
      if (!window.AurexalisCore) {
        return fallback;
      }
      return AurexalisCore.getString(`remotefs.${name}`, fallback);
    }

    function savePref(name, value) {
      if (!window.AurexalisCore) {
        return;
      }
      AurexalisCore.setString(`remotefs.${name}`, value);
    }

    function addField(labelText, value, onInput) {
      const row = xul("hbox");
      row.className = "ax-rf-field";
      const label = xul("label");
      label.className = "ax-rf-field-label";
      label.textContent = labelText;
      const input = xul("textbox");
      input.className = "ax-rf-field-input";
      input.setAttribute("flex", "1");
      input.value = value;
      input.addEventListener("input", () => onInput(input.value));
      row.appendChild(label);
      row.appendChild(input);
      host.appendChild(row);
      return input;
    }

    const protocolRow = xul("hbox");
    protocolRow.className = "ax-rf-field";
    const protocolLabel = xul("label");
    protocolLabel.className = "ax-rf-field-label";
    protocolLabel.textContent = "Protocolo";
    const protocolList = xul("menulist");
    protocolList.className = "ax-rf-field-input";
    const popup = xul("menupopup");
    for (const option of [
      ["sftp", "SFTP (22)"],
      ["ftp", "FTP (21)"],
      ["ftps", "FTPS (990 impl.)"],
    ]) {
      const item = xul("menuitem");
      item.setAttribute("label", option[1]);
      item.setAttribute("value", option[0]);
      popup.appendChild(item);
    }
    protocolList.appendChild(popup);
    protocolList.value = pref("protocol", "sftp");
    protocolList.addEventListener("command", () => savePref("protocol", protocolList.value));
    protocolRow.appendChild(protocolLabel);
    protocolRow.appendChild(protocolList);
    host.appendChild(protocolRow);

    const hostInput = addField("Host", pref("host", ""), (v) => savePref("host", v));
    const userInput = addField("Usuario", pref("user", ""), (v) => savePref("user", v));
    const pathInput = addField("Ruta", pref("path", "/"), (v) => savePref("path", v));

    const hint = xul("description");
    hint.className = "ax-rf-hint";
    hint.textContent =
      "Contrasena via env: AUREXALIS_SFTP_PASS, AUREXALIS_FTP_PASS o AUREXALIS_FTPS_PASS";
    host.appendChild(hint);

    function runRemoteList() {
      if (!window.AurexalisCore) {
        Services.prompt.alert(null, "Aurexalis", "Nucleo Aurexalis no cargado.");
        return;
      }
      const hostValue = hostInput.value.trim();
      const userValue = userInput.value.trim();
      const pathValue = pathInput.value.trim() || "/";
      if (!hostValue || !userValue) {
        Services.prompt.alert(null, "Aurexalis — RemoteFS", "Completa host y usuario.");
        return;
      }
      savePref("host", hostValue);
      savePref("user", userValue);
      savePref("path", pathValue);
      savePref("protocol", protocolList.value);
      try {
        AurexalisCore.runShell([
          "remotefs",
          "list",
          "--protocol",
          protocolList.value,
          "--host",
          hostValue,
          "--user",
          userValue,
          "--path",
          pathValue,
        ]);
        if (window.AurexalisSound) {
          AurexalisSound.play("panel");
        }
      } catch (error) {
        Services.prompt.alert(null, "Aurexalis", String(error));
      }
    }

    while (actions.firstChild) {
      actions.firstChild.remove();
    }

    const listBtn = xul("button");
    listBtn.className = "ax-panel-action";
    listBtn.textContent = "Listar directorio (CLI)";
    listBtn.onclick = runRemoteList;
    actions.appendChild(listBtn);

    const downloadsBtn = xul("button");
    downloadsBtn.className = "ax-panel-action ax-panel-action-secondary";
    downloadsBtn.textContent = "Abrir descargas";
    downloadsBtn.onclick = openDownloads;
    actions.appendChild(downloadsBtn);

    const helpBtn = xul("button");
    helpBtn.className = "ax-panel-action ax-panel-action-secondary";
    helpBtn.textContent = "Ayuda CLI";
    helpBtn.onclick = () => {
      Services.prompt.alert(
        null,
        "Aurexalis — RemoteFS",
        "Listar:\n  aurexalis remotefs list --protocol ftp --host H --user U --path /\n\n" +
          "Descargar:\n  aurexalis remotefs get --protocol sftp --host H --user U --remote /f --local C:\\tmp\\f\n\n" +
          "Credenciales: AUREXALIS_SFTP_PASS | AUREXALIS_FTP_PASS | AUREXALIS_FTPS_PASS"
      );
    };
    actions.appendChild(helpBtn);

    panel.hidden = false;
    setActive(button);
  }

  function importerPanel() {
    return {
      title: "Importador local",
      rows: [
        ["Datos", "Marcadores, historial, cookies"],
        ["Origen", "Chrome, Brave u Opera instalados"],
        ["Apply", "import apply → places.sqlite (navegador cerrado)"],
      ],
      action: "Exportar sin contrasenas",
      command: () => runImport(false),
      extraActions: [
        {
          label: "Exportar con contrasenas",
          command: () => {
            const ok = Services.prompt.confirm(
              null,
              "Aurexalis — contrasenas",
              "Se exportaran contrasenas a un JSON local en tu perfil. " +
                "No se envia nada a Internet. ¿Continuar?"
            );
            if (ok) {
              runImport(true);
            }
          },
        },
        {
          label: "Aplicar al perfil Gecko",
          command: () => {
            const ok = Services.prompt.confirm(
              null,
              "Aurexalis — import apply",
              "Cierra Aurexalis antes de importar. ¿Aplicar marcadores e historial?"
            );
            if (ok) {
              runShellApply();
            }
          },
        },
      ],
    };
  }

  function runShellApply() {
    if (!window.AurexalisCore) {
      return;
    }
    try {
      AurexalisCore.runShell(["import", "apply"]);
      if (window.AurexalisSound) {
        AurexalisSound.play("panel");
      }
    } catch (error) {
      Services.prompt.alert(null, "Aurexalis", String(error));
    }
  }

  function settingsPanel() {
    return { title: "Ajustes Aurexalis", settings: true };
  }

  function toggleBlocker() {
    if (!window.AurexalisCore) {
      return;
    }
    const next = !AurexalisCore.getBool("blocker.enabled", true);
    AurexalisCore.setBool("blocker.enabled", next);
    if (window.AurexalisSound) {
      AurexalisSound.play("panel");
    }
    const state = next ? "activado" : "desactivado";
    try {
      Services.prompt.alert(null, "Aurexalis — Bloqueador", `Bloqueador ${state}.`);
    } catch (_error) {
      console.info(`[AurexalisSidebar] Bloqueador ${state}`);
    }
  }

  function setActive(button) {
    for (const item of document.querySelectorAll(".ax-sidebar-button")) {
      item.classList.remove("ax-active");
      item.removeAttribute("checked");
    }
    button.classList.add("ax-active");
    button.setAttribute("checked", "true");
  }

  function showInfoPanel(item, button) {
    const panel = document.getElementById("ax-sidebar-panel");
    const title = panel.querySelector(".ax-panel-header");
    const body = panel.querySelector(".ax-panel-body");
    const rows = panel.querySelector(".ax-panel-rows");
    const actions = panel.querySelector(".ax-panel-actions");
    const content = item.panel();

    title.textContent = content.title;
    while (rows.firstChild) {
      rows.firstChild.remove();
    }
    while (actions.firstChild) {
      actions.firstChild.remove();
    }

    if (content.explorer) {
      buildRemoteExplorer(panel, body, button);
      return;
    }

    if (content.settings && window.AurexalisSettingsPanel) {
      rows.hidden = true;
      actions.hidden = true;
      let settingsHost = panel.querySelector(".ax-settings-host");
      if (!settingsHost) {
        settingsHost = xul("vbox");
        settingsHost.className = "ax-settings-host";
        body.insertBefore(settingsHost, rows);
      }
      settingsHost.hidden = false;
      AurexalisSettingsPanel.build(settingsHost);
    } else {
      rows.hidden = false;
      actions.hidden = false;
      const settingsHost = panel.querySelector(".ax-settings-host");
      if (settingsHost) {
        settingsHost.hidden = true;
      }
      const rfHost = panel.querySelector(".ax-rf-explorer");
      if (rfHost) {
        rfHost.hidden = true;
      }

      for (const row of content.rows) {
        const line = xul("hbox");
        line.className = "ax-panel-row";
        const key = xul("label");
        key.className = "ax-panel-row-key";
        key.textContent = row[0];
        const value = xul("description");
        value.className = "ax-panel-row-value";
        value.textContent = row[1];
        line.appendChild(key);
        line.appendChild(value);
        rows.appendChild(line);
      }

      const primary = xul("button");
      primary.className = "ax-panel-action";
      primary.textContent = content.action;
      primary.onclick = () => content.command();
      actions.appendChild(primary);

      if (content.extraActions) {
        for (const extra of content.extraActions) {
          const btn = xul("button");
          btn.className = "ax-panel-action ax-panel-action-secondary";
          btn.textContent = extra.label;
          btn.onclick = () => extra.command();
          actions.appendChild(btn);
        }
      }
    }

    panel.hidden = false;
    setActive(button);
  }

  function buildPanel() {
    const panel = xul("vbox");
    panel.id = "ax-sidebar-panel";
    panel.hidden = true;

    const title = xul("label");
    title.className = "ax-panel-header";

    const body = xul("vbox");
    body.className = "ax-panel-body";
    const rows = xul("vbox");
    rows.className = "ax-panel-rows";
    const actions = xul("vbox");
    actions.className = "ax-panel-actions";

    body.appendChild(rows);
    body.appendChild(actions);
    panel.appendChild(title);
    panel.appendChild(body);
    return panel;
  }

  function buildSidebar() {
    const sidebar = xul("vbox");
    sidebar.id = "ax-sidebar";

    const top = xul("vbox");
    top.className = "ax-sidebar-section";
    const bottom = xul("vbox");
    bottom.className = "ax-sidebar-section";
    const spacer = xul("spacer");
    spacer.className = "ax-sidebar-spacer";
    spacer.setAttribute("flex", "1");

    for (const item of items) {
      const button = xul("toolbarbutton");
      button.id = `ax-sidebar-${item.id}`;
      button.className = "ax-sidebar-button";
      button.setAttribute("label", item.label);
      button.setAttribute("tooltiptext", item.title);
      button.addEventListener("command", () => {
        if (item.panel) {
          showInfoPanel(item, button);
          if (window.AurexalisSound) {
            AurexalisSound.play("panel");
          }
          return;
        }
        const panel = document.getElementById("ax-sidebar-panel");
        if (panel) {
          panel.hidden = true;
        }
        setActive(button);
        item.action();
      });

      if (["blocker", "passwords", "settings"].includes(item.id)) {
        bottom.appendChild(button);
      } else {
        top.appendChild(button);
      }
    }

    sidebar.appendChild(top);
    sidebar.appendChild(spacer);
    sidebar.appendChild(bottom);
    return sidebar;
  }

  function mount() {
    const browser = document.getElementById("browser");
    if (!browser || document.getElementById("ax-sidebar")) {
      return;
    }

    browser.insertBefore(buildSidebar(), browser.firstChild);
    browser.insertBefore(buildPanel(), browser.children[1] || null);
    console.info("[AurexalisSidebar] Mounted");
  }

  if (document.readyState === "complete") {
    mount();
  } else {
    window.addEventListener("load", mount, { once: true });
  }
})();
