// Aurexalis sidebar scaffold for Firefox/Floorp chrome UI experiments.
// Provides a GX-like vertical dock with placeholders for native Aurexalis modules.

(function initAurexalisSidebar() {
  "use strict";

  if (window.__aurexalisSidebarLoaded) {
    return;
  }
  window.__aurexalisSidebarLoaded = true;

  const items = [
    { id: "home", label: "AX", title: "Aurexalis Home", action: () => openTab("about:home") },
    { id: "gx", label: "GX", title: "GX Corner", action: () => openTab("https://gxcorner.games/") },
    { id: "remote", label: "RF", title: "Remote Files", panel: remoteFilesPanel },
    { id: "bookmarks", label: "BM", title: "Bookmarks", action: toggleBookmarks },
    { id: "downloads", label: "DL", title: "Downloads", action: openDownloads },
    { id: "importer", label: "IM", title: "Profile Importer", panel: importerPanel },
    { id: "passwords", label: "PW", title: "Passwords", action: () => openTab("about:logins") },
    { id: "settings", label: "ST", title: "Settings", action: () => openTab("about:preferences") },
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

  function remoteFilesPanel() {
    return {
      title: "Remote Files",
      rows: [
        ["Backend", "aurexalis-remotefs"],
        ["Protocolos", "SFTP / FTP / FTPS"],
        ["Estado", "cola de transferencias y credenciales seguras en Rust"],
      ],
      action: "Abrir descargas",
      command: openDownloads,
    };
  }

  function importerPanel() {
    return {
      title: "Profile Importer",
      rows: [
        ["SQLite", "Cookies / Login Data / History / Favicons"],
        ["JSON", "Bookmarks / Preferences / Secure Preferences / Local State"],
        ["Claves", "DPAPI Windows; Linux Secret Service/KWallet en adaptador"],
      ],
      action: "Abrir contrasenas",
      command: () => openTab("about:logins"),
    };
  }

  function setActive(button) {
    for (const item of document.querySelectorAll(".ax-sidebar-button")) {
      item.classList.remove("ax-active");
      item.removeAttribute("checked");
    }
    button.classList.add("ax-active");
    button.setAttribute("checked", "true");
  }

  function showPanel(item, button) {
    const panel = document.getElementById("ax-sidebar-panel");
    const title = panel.querySelector(".ax-panel-header");
    const rows = panel.querySelector(".ax-panel-rows");
    const action = panel.querySelector(".ax-panel-action");
    const content = item.panel();

    title.textContent = content.title;
    while (rows.firstChild) {
      rows.firstChild.remove();
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
    action.textContent = content.action;
    action.onclick = () => content.command();
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
    const action = xul("button");
    action.className = "ax-panel-action";

    body.appendChild(rows);
    body.appendChild(action);
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
          showPanel(item, button);
          return;
        }
        const panel = document.getElementById("ax-sidebar-panel");
        if (panel) {
          panel.hidden = true;
        }
        setActive(button);
        item.action();
      });

      if (["passwords", "settings"].includes(item.id)) {
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
