// Aurexalis early profile prefs. Use only in a dedicated test profile.

user_pref("toolkit.legacyUserProfileCustomizations.stylesheets", true);
user_pref("browser.tabs.tabmanager.enabled", true);
user_pref("browser.compactmode.show", true);
user_pref("browser.uidensity", 1);

// Prefer Aurexalis home over Firefox Activity Stream (URL se completa al instalar).
user_pref("browser.newtabpage.enabled", false);
user_pref("browser.startup.page", 1);
user_pref("browser.newtabpage.activity-stream.feeds.section.topstories", false);
user_pref("browser.aboutHomeSnippets.updateUrl", "");

// Aurexalis — sonidos y UI (editables desde panel ST).
user_pref("aurexalis.sounds.enabled", true);
user_pref("aurexalis.sounds.master", 22);
user_pref("aurexalis.sounds.click.enabled", true);
user_pref("aurexalis.sounds.click.volume", 85);
user_pref("aurexalis.sounds.hover.enabled", true);
user_pref("aurexalis.sounds.hover.volume", 55);
user_pref("aurexalis.sounds.key.enabled", true);
user_pref("aurexalis.sounds.key.volume", 40);
user_pref("aurexalis.sounds.ambient.enabled", true);
user_pref("aurexalis.sounds.ambient.volume", 12);
user_pref("aurexalis.sounds.panel.enabled", true);
user_pref("aurexalis.sounds.panel.volume", 70);
user_pref("aurexalis.ui.animations", true);

// Bloqueador Aurexalis (Gecko ETP + panel ST / ajustes).
user_pref("aurexalis.blocker.enabled", true);
user_pref("aurexalis.blocker.level", "standard");
user_pref("aurexalis.blocker.cosmetic", true);

// shell.path y settings.url se escriben al instalar.

// Identidad Aurexalis sobre nucleo Gecko.
user_pref("svg.context-properties.content.enabled", true);
user_pref("app.feedback.baseURL", "");
user_pref("browser.preferences.moreFromMozilla", false);
user_pref("browser.shell.checkDefaultBrowser", true);
user_pref("floorp.design.configs", "{\"uiCustomization\":{\"disableFloorpStart\":true}}");
