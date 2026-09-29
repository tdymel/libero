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

// Tells the page about a notification `use_system_notification` showed here,
// so its `on_click` runs. Pages without that notification ignore the message.
const tell = (notification, name) => {
  const libero = notification.data?.libero;
  if (!libero) return Promise.resolve([]);
  return self.clients.matchAll({ type: 'window', includeUncontrolled: true }).then((tabs) => {
    for (const tab of tabs) tab.postMessage({ libero, event: name });
    return tabs;
  });
};

// Focuses an open tab of the app, else opens one.
self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  const url = event.notification.data?.url ?? '/';
  event.waitUntil(
    tell(event.notification, 'click')
      // A close from here fires no `notificationclose`, so the page forgets it now.
      .then(() => tell(event.notification, 'close'))
      .then(() => self.clients.matchAll({ type: 'window', includeUncontrolled: true }))
      .then((tabs) => {
        const tab = tabs.find((client) => 'focus' in client);
        return tab ? tab.focus() : self.clients.openWindow(url);
      }),
  );
});

self.addEventListener('notificationclose', (event) => {
  event.waitUntil(tell(event.notification, 'close'));
});
