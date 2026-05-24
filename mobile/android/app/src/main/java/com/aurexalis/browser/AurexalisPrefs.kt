package com.aurexalis.browser

import android.content.Context
import org.mozilla.geckoview.ContentBlocking

/** Preferencias Aurexalis en Android (paridad con prefs desktop aurexalis.*). */
object AurexalisPrefs {
    private const val STORE = "aurexalis_prefs"

    private fun prefs(ctx: Context) =
        ctx.applicationContext.getSharedPreferences(STORE, Context.MODE_PRIVATE)

    fun getBool(ctx: Context, name: String, fallback: Boolean): Boolean =
        prefs(ctx).getBoolean("aurexalis.$name", fallback)

    fun setBool(ctx: Context, name: String, value: Boolean) {
        prefs(ctx).edit().putBoolean("aurexalis.$name", value).apply()
    }

    fun getInt(ctx: Context, name: String, fallback: Int): Int =
        prefs(ctx).getInt("aurexalis.$name", fallback)

    fun setInt(ctx: Context, name: String, value: Int) {
        prefs(ctx).edit().putInt("aurexalis.$name", value.coerceIn(0, 100)).apply()
    }

    fun getString(ctx: Context, name: String, fallback: String): String =
        prefs(ctx).getString("aurexalis.$name", fallback) ?: fallback

    fun setString(ctx: Context, name: String, value: String) {
        prefs(ctx).edit().putString("aurexalis.$name", value).apply()
    }

  /** Construye ContentBlocking segun prefs de bloqueador Aurexalis. */
    fun contentBlockingSettings(ctx: Context): ContentBlocking.Settings {
        val enabled = getBool(ctx, "blocker.enabled", true)
        val level = getString(ctx, "blocker.level", "standard")

        if (!enabled || level == "off") {
            return ContentBlocking.Settings.Builder()
                .antiTracking(ContentBlocking.AntiTracking.NONE)
                .safeBrowsing(ContentBlocking.SafeBrowsing.DEFAULT)
                .build()
        }

        val antiTracking =
            if (level == "strict") {
                ContentBlocking.AntiTracking.STRICT or ContentBlocking.AntiTracking.DEFAULT
            } else {
                ContentBlocking.AntiTracking.DEFAULT
            }

        val builder = ContentBlocking.Settings.Builder()
            .antiTracking(antiTracking)
            .safeBrowsing(ContentBlocking.SafeBrowsing.DEFAULT)
            .cookieBehavior(ContentBlocking.CookieBehavior.ACCEPT_NON_TRACKERS)
            .strictSocialTrackingProtection(level == "strict")

        return builder.build()
    }
}
