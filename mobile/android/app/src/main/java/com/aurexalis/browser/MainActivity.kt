package com.aurexalis.browser

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.view.KeyEvent
import android.view.inputmethod.EditorInfo
import android.widget.Toast
import androidx.activity.addCallback
import androidx.appcompat.app.AppCompatActivity
import com.aurexalis.browser.databinding.ActivityMainBinding
import org.mozilla.geckoview.AllowOrDeny
import org.mozilla.geckoview.GeckoResult
import org.mozilla.geckoview.GeckoSession
import org.mozilla.geckoview.GeckoSession.NavigationDelegate
import org.mozilla.geckoview.GeckoSession.ProgressDelegate

/** Navegador Aurexalis para Android (GeckoView + home/settings embebidos). */
class MainActivity : AppCompatActivity() {
    private lateinit var binding: ActivityMainBinding
    private lateinit var session: GeckoSession

    // GeckoView no carga file:///android_asset/ (solo WebView); usar resource://.
    private val assetBase = "resource://android/assets/aurexalis/"
    private val homeUrl = "${assetBase}home/index.html"
    private val settingsUrl = "${assetBase}settings/index.html"

    private var canGoBack = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        session = GeckoSession()
        openSession()

        session.navigationDelegate = object : NavigationDelegate {
            override fun onLocationChange(
                session: GeckoSession,
                url: String?,
                permissions: MutableList<GeckoSession.PermissionDelegate.ContentPermission>,
                isSameDocument: Boolean,
            ) {
                if (!url.isNullOrBlank()) {
                    binding.urlInput.setText(url)
                    binding.urlInput.setSelection(url.length)
                    canGoBack = url != homeUrl && url != settingsUrl
                }
            }

            override fun onLoadRequest(
                session: GeckoSession,
                request: NavigationDelegate.LoadRequest,
            ): GeckoResult<AllowOrDeny>? {
                val target = request.uri
                if (target.startsWith("aurexalis://pref/")) {
                    handlePrefUri(Uri.parse(target))
                    return GeckoResult.deny()
                }
                if (isEmbeddedAssetUrl(target)) {
                    return GeckoResult.allow()
                }
                if (
                    target.startsWith("http://", ignoreCase = true) ||
                    target.startsWith("https://", ignoreCase = true) ||
                    target.startsWith("about:")
                ) {
                    return GeckoResult.allow()
                }
                if (target.startsWith("mailto:", ignoreCase = true) ||
                    target.startsWith("tel:", ignoreCase = true)
                ) {
                    return try {
                        startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(target)))
                        GeckoResult.deny()
                    } catch (_: Exception) {
                        GeckoResult.deny()
                    }
                }
                return GeckoResult.deny()
            }
        }

        session.progressDelegate = object : ProgressDelegate {
            override fun onPageStart(session: GeckoSession, url: String) {
                binding.progressBar.isIndeterminate = true
                binding.progressBar.visibility = android.view.View.VISIBLE
            }

            override fun onPageStop(session: GeckoSession, success: Boolean) {
                binding.progressBar.visibility = android.view.View.GONE
                injectSettingsBridgeIfNeeded(session, urlFromSession())
            }
        }

        onBackPressedDispatcher.addCallback(this) {
            if (canGoBack) {
                session.goBack()
            } else {
                finish()
            }
        }

        binding.btnBack.setOnClickListener { session.goBack() }
        binding.btnForward.setOnClickListener { session.goForward() }
        binding.btnReload.setOnClickListener { session.reload() }
        binding.btnHome.setOnClickListener { loadUrl(homeUrl) }
        binding.btnSettings.setOnClickListener { loadUrl(settingsUrl) }

        binding.urlInput.setOnEditorActionListener { _, actionId, event ->
            val enter =
                actionId == EditorInfo.IME_ACTION_GO ||
                    actionId == EditorInfo.IME_ACTION_DONE ||
                    (event?.keyCode == KeyEvent.KEYCODE_ENTER && event.action == KeyEvent.ACTION_DOWN)
            if (enter) {
                navigateFromBar()
                true
            } else {
                false
            }
        }

        binding.btnGo.setOnClickListener { navigateFromBar() }

        when (val data = intent?.dataString) {
            null -> loadUrl(homeUrl)
            else -> loadUrl(normalizeInput(data))
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        intent.dataString?.let { loadUrl(normalizeInput(it)) }
    }

    private fun openSession() {
        session.open(AurexalisApp.geckoRuntime(application))
        binding.geckoView.setSession(session)
    }

    private fun urlFromSession(): String? = binding.urlInput.text?.toString()

    /** Aplica cambios de prefs desde la pagina de ajustes (aurexalis://pref/set). */
    private fun handlePrefUri(uri: Uri) {
        if (uri.host != "pref" || uri.path != "/set") {
            return
        }
        val type = uri.getQueryParameter("type") ?: return
        val name = uri.getQueryParameter("name") ?: return
        val value = uri.getQueryParameter("value") ?: return

        when (type) {
            "bool" -> AurexalisPrefs.setBool(this, name, value == "1" || value.equals("true", true))
            "int" -> AurexalisPrefs.setInt(this, name, value.toIntOrNull() ?: 0)
            "string" -> AurexalisPrefs.setString(this, name, value)
        }

        if (name.startsWith("blocker.")) {
            Toast.makeText(this, R.string.blocker_updated, Toast.LENGTH_SHORT).show()
            restartSessionForBlocker()
        }
    }

    private fun restartSessionForBlocker() {
        val current = urlFromSession() ?: settingsUrl
        session.close()
        session = GeckoSession()
        AurexalisApp.resetRuntime(application)
        openSession()
        loadUrl(current)
    }

    /** Inyecta AurexalisPrefsBridge en la pagina de ajustes embebida. */
    private fun injectSettingsBridgeIfNeeded(session: GeckoSession, url: String?) {
        if (url == null || !url.contains("/settings/index.html")) {
            return
        }
        val p = applicationContext
        val script =
            """
            (function(){
              if (window.AurexalisPrefsBridge) return;
              var store = {
                "sounds.enabled": ${AurexalisPrefs.getBool(p, "sounds.enabled", true)},
                "sounds.master": ${AurexalisPrefs.getInt(p, "sounds.master", 22)},
                "sounds.click.enabled": ${AurexalisPrefs.getBool(p, "sounds.click.enabled", true)},
                "sounds.click.volume": ${AurexalisPrefs.getInt(p, "sounds.click.volume", 85)},
                "sounds.hover.enabled": ${AurexalisPrefs.getBool(p, "sounds.hover.enabled", true)},
                "sounds.hover.volume": ${AurexalisPrefs.getInt(p, "sounds.hover.volume", 55)},
                "sounds.key.enabled": ${AurexalisPrefs.getBool(p, "sounds.key.enabled", true)},
                "sounds.key.volume": ${AurexalisPrefs.getInt(p, "sounds.key.volume", 40)},
                "sounds.ambient.enabled": ${AurexalisPrefs.getBool(p, "sounds.ambient.enabled", true)},
                "sounds.ambient.volume": ${AurexalisPrefs.getInt(p, "sounds.ambient.volume", 12)},
                "sounds.panel.enabled": ${AurexalisPrefs.getBool(p, "sounds.panel.enabled", true)},
                "sounds.panel.volume": ${AurexalisPrefs.getInt(p, "sounds.panel.volume", 70)},
                "ui.animations": ${AurexalisPrefs.getBool(p, "ui.animations", true)},
                "blocker.enabled": ${AurexalisPrefs.getBool(p, "blocker.enabled", true)},
                "blocker.level": "${AurexalisPrefs.getString(p, "blocker.level", "standard")}",
                "blocker.cosmetic": ${AurexalisPrefs.getBool(p, "blocker.cosmetic", true)}
              };
              function beacon(type, name, value) {
                var img = new Image();
                img.src = "aurexalis://pref/set?type=" + encodeURIComponent(type) +
                  "&name=" + encodeURIComponent(name) + "&value=" + encodeURIComponent(String(value));
              }
              window.AurexalisPrefsBridge = {
                getBool: function(n,f){ return Object.prototype.hasOwnProperty.call(store,n) ? !!store[n] : f; },
                setBool: function(n,v){ store[n]=!!v; beacon("bool", n, v ? "1" : "0"); },
                getInt: function(n,f){ return Object.prototype.hasOwnProperty.call(store,n) ? Number(store[n]) : f; },
                setInt: function(n,v){ store[n]=v; beacon("int", n, v); },
                getString: function(n,f){ return Object.prototype.hasOwnProperty.call(store,n) ? String(store[n]) : f; },
                setString: function(n,v){ store[n]=v; beacon("string", n, v); }
              };
              window.dispatchEvent(new CustomEvent("aurexalis-prefs-ready"));
            })();
            """.trimIndent()
        session.loadUri("javascript:${Uri.encode(script)}")
    }

    private fun navigateFromBar() {
        val raw = binding.urlInput.text?.toString()?.trim().orEmpty()
        if (raw.isEmpty()) {
            Toast.makeText(this, R.string.hint_empty_url, Toast.LENGTH_SHORT).show()
            return
        }
        loadUrl(normalizeInput(raw))
    }

    private fun isEmbeddedAssetUrl(url: String): Boolean {
        return url.startsWith(assetBase) ||
            url.startsWith("file:///android_asset/aurexalis/")
    }

    /** Convierte URIs legacy file:///android_asset/ a resource:// para GeckoView. */
    private fun resolveLoadUrl(url: String): String {
        if (url.startsWith("file:///android_asset/")) {
            return url.replaceFirst(
                "file:///android_asset/",
                "resource://android/assets/",
            )
        }
        return url
    }

    private fun normalizeInput(value: String): String {
        if (
            value.startsWith("http://", ignoreCase = true) ||
            value.startsWith("https://", ignoreCase = true) ||
            value.startsWith("resource://") ||
            value.startsWith("file://")
        ) {
            return resolveLoadUrl(value)
        }
        return if (value.contains(".") && !value.contains(" ")) {
            "https://$value"
        } else {
            "https://duckduckgo.com/?q=${Uri.encode(value)}"
        }
    }

    private fun loadUrl(url: String) {
        val resolved = resolveLoadUrl(url)
        session.loadUri(resolved)
        binding.urlInput.setText(resolved)
        binding.urlInput.setSelection(resolved.length)
    }
}
