// A sample service worker for `use_push_subscription`. The app owns its worker:
// copy this into the app's `public/` and adapt it. Libero ships no worker.

// The app's server decides the payload; this sample expects `{ title, body, url }`.
self.addEventListener('push', (event) => {
  const message = event.data?.json() ?? {};
  event.waitUntil(
    self.registration.showNotification(message.title ?? 'New message', {
      body: message.body,
      data: { url: message.url ?? '/' },
    }),
  );
});

// Focuses an open tab of the app, else opens one.
self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  const url = event.notification.data?.url ?? '/';
  event.waitUntil(
    self.clients.matchAll({ type: 'window', includeUncontrolled: true }).then((tabs) => {
      const tab = tabs.find((client) => 'focus' in client);
      return tab ? tab.focus() : self.clients.openWindow(url);
    }),
  );
});
