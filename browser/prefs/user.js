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

