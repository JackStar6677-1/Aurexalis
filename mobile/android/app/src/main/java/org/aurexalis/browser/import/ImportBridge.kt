package org.aurexalis.browser.import

/**
 * Puente futuro hacia `aurexalis-importer` (UniFFI/JNI).
 * v0.5: stub — importacion nativa Chromium pendiente.
 */
object ImportBridge {
    const val STATUS_NOT_IMPLEMENTED = "NOT_IMPLEMENTED"

    /** Lista perfiles Chromium detectables en el dispositivo. */
    fun listProfiles(): List<String> = emptyList()

    /** Aplica snapshot auditado al perfil GeckoView (requiere consentimiento en UI). */
    fun applyAudit(profilePath: String, auditJsonPath: String, surfaces: List<String>): String =
        STATUS_NOT_IMPLEMENTED
}
