console.log('Service worker starting');

self.addEventListener('activate', (event: any) => {
  event.waitUntil((self as any).clients.claim());
});

self.addEventListener('fetch', async (event: any) => {
  const url = new URL(event.request.url);

  // check if the request is an IPFS API call
  if (url.pathname.startsWith('/api/v0/')) {
    console.log('IPFS API call detected; url:', url);
    const clients = (self as any).clients;
    // const clients = await (self as any).clients.matchAll();
    const response = new Promise<Response>(async (resolve) => {
      const messageChannel = new MessageChannel();
      // handle response from main thread
      messageChannel.port1.onmessage = (event) => {
        if (event.data.error) {
          resolve(new Response(event.data.error, { status: 500 }));
        } else {
          resolve(new Response(event.data.response, { status: 200 }));
        }
      };
      // serialize request manually (not serializable)
      const requestData = {
        method: event.request.method,
        url: event.request.url,
        // headers: event.request.headers,
        body: event.request.body,
      };
      const c = await clients.matchAll();
      if (c && c.length > 0) {
        c[0].postMessage({
            type: 'IPFS_REQUEST',
            request: requestData,
          },
          [messageChannel.port2]
        );
      }
    });
    event.respondWith(response); // TODO: figure out why this doesn't work
    // event.respondWith(new Promise<Response>((resolve) => resolve(new Response(JSON.stringify({ Entries: null }), { status: 200 }))));
  }
});
