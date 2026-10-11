package dev.libero.fcm

import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage

class LiberoFcmService : FirebaseMessagingService() {
    override fun onNewToken(token: String) = LiberoFcmEvents.token(token)

    override fun onMessageReceived(message: RemoteMessage) =
        LiberoFcmEvents.message(message.data, message.notification?.title, message.notification?.body)
}
