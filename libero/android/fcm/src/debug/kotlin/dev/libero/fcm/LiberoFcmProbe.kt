package dev.libero.fcm

/** Debug builds only (e2e): a new token and a message down the service's own path. */
object LiberoFcmProbe {
    @JvmStatic
    fun run(token: String, data: String) {
        LiberoFcmEvents.token(token)
        LiberoFcmEvents.message(mapOf("probe" to data), "Probe", data)
    }
}
