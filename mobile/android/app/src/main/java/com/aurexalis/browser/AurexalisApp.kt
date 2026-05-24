package com.aurexalis.browser

import android.app.Application
import org.mozilla.geckoview.GeckoRuntime
import org.mozilla.geckoview.GeckoRuntimeSettings

/** Mantiene un unico GeckoRuntime con bloqueo de anuncios segun prefs Aurexalis. */
class AurexalisApp : Application() {
    companion object {
        @Volatile
        private var runtime: GeckoRuntime? = null

        fun geckoRuntime(app: Application): GeckoRuntime {
            return runtime ?: synchronized(this) {
                runtime ?: createRuntime(app).also { runtime = it }
            }
        }

        /** Recrea el runtime tras cambiar prefs de bloqueador (Android). */
        fun resetRuntime(app: Application) {
            synchronized(this) {
                runtime?.shutdown()
                runtime = null
            }
            geckoRuntime(app)
        }

        private fun createRuntime(app: Application): GeckoRuntime {
            val settings = GeckoRuntimeSettings.Builder()
                .contentBlocking(AurexalisPrefs.contentBlockingSettings(app))
                .build()
            return GeckoRuntime.create(app, settings)
        }
    }
}
