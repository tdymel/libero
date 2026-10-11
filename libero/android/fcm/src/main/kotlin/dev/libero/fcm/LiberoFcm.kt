package dev.libero.fcm

import android.app.Activity
import com.google.firebase.FirebaseApp
import com.google.firebase.FirebaseOptions
import com.google.firebase.messaging.FirebaseMessaging

/** Called from Rust over manganis `#[ffi]`; answers come back through [LiberoFcmEvents]. */
class LiberoFcm(private val activity: Activity) {
    /** Starts Firebase with the app's options once and asks for the token. */
    fun register(applicationId: String, apiKey: String, projectId: String, senderId: String) {
        try {
            if (FirebaseApp.getApps(activity).isEmpty()) {
                val options = FirebaseOptions.Builder()
                    .setApplicationId(applicationId)
                    .setApiKey(apiKey)
                    .setProjectId(projectId)
                    .setGcmSenderId(senderId)
                    .build()
                FirebaseApp.initializeApp(activity.applicationContext, options)
            }
            FirebaseMessaging.getInstance().token.addOnCompleteListener { task ->
                if (task.isSuccessful) {
                    LiberoFcmEvents.token(task.result)
                } else {
                    LiberoFcmEvents.error(task.exception?.toString() ?: "no token")
                }
            }
        } catch (e: Exception) {
            LiberoFcmEvents.error(e.toString())
        }
    }

    /** Hands Rust what arrived before its library was loaded. */
    fun flush() = LiberoFcmEvents.flush()

    /** Debug builds only: runs `LiberoFcmProbe` from `src/debug`, as the service would. */
    fun probe(token: String, data: String) {
        try {
            Class.forName("dev.libero.fcm.LiberoFcmProbe")
                .getMethod("run", String::class.java, String::class.java)
                .invoke(null, token, data)
        } catch (e: ClassNotFoundException) {
            LiberoFcmEvents.error("no probe in a release build")
        }
    }
}

/** Kotlin to Rust: each event calls into Rust, or waits here until its library is loaded. */
object LiberoFcmEvents {
    private val pending = mutableListOf<() -> Unit>()

    @JvmStatic private external fun onToken(token: String)

    @JvmStatic private external fun onMessage(data: Array<String>, title: String?, body: String?)

    @JvmStatic private external fun onError(error: String)

    private fun send(event: () -> Unit) {
        synchronized(pending) {
            try {
                event()
            } catch (e: UnsatisfiedLinkError) {
                pending.add(event)
            }
        }
    }

    fun flush() {
        val events = synchronized(pending) { pending.toList().also { pending.clear() } }
        events.forEach { send(it) }
    }

    fun token(token: String) = send { onToken(token) }

    /** `data` goes over as key, value, key, value. */
    fun message(data: Map<String, String>, title: String?, body: String?) {
        val pairs = data.flatMap { listOf(it.key, it.value) }.toTypedArray()
        send { onMessage(pairs, title, body) }
    }

    fun error(error: String) = send { onError(error) }
}
