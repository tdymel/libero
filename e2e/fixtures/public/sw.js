// `use_push_subscription`'s fixture worker: registering it is all the test needs.
self.addEventListener('push', (event) => {
  event.waitUntil(self.registration.showNotification('Fixture push'));
});
